//! Private single-process journal. A complete synced frame precedes every effect.
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

pub fn private_directory(path: &Path) -> Result<(), &'static str> {
    fs::create_dir_all(path).map_err(|_| "storage_unavailable")?;
    if is_link(path)? {
        return Err("storage_not_private");
    }
    #[cfg(windows)]
    {
        use std::{os::windows::ffi::OsStrExt, ptr};
        use windows_sys::Win32::{
            Foundation::LocalFree,
            Security::{
                Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW,
                DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, SetFileSecurityW,
            },
        };
        let sddl: Vec<u16> = "D:P(A;OICI;FA;;;OW)\0".encode_utf16().collect();
        let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut descriptor = ptr::null_mut();
        // SAFETY: buffers are NUL terminated, descriptor is owned until LocalFree.
        let success = unsafe {
            if ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            ) == 0
            {
                return Err("storage_not_private");
            }
            let result = SetFileSecurityW(
                name.as_ptr(),
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                descriptor,
            );
            LocalFree(descriptor);
            result
        };
        if success == 0 {
            return Err("storage_not_private");
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| "storage_not_private")?;
    }
    Ok(())
}

pub struct Journal {
    file: File,
    path: std::path::PathBuf,
    bytes: u64,
    failed: bool,
}
impl Journal {
    pub fn open(root: &Path, name: &str) -> Result<(Self, Vec<serde_json::Value>), &'static str> {
        private_directory(root)?;
        let path = root.join(name);
        if path.exists() && is_link(&path)? {
            return Err("storage_not_private");
        }
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).read(true).write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(4);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&path)
            .map_err(|_| "storage_unavailable_or_locked")?;
        if file.metadata().map_err(|_| "storage_unavailable")?.len() > 64 * 1024 * 1024 {
            return Err("storage_full");
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|_| "storage_unavailable")?;
        let valid = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        let mut records = Vec::new();
        for line in bytes[..valid]
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let frame: serde_json::Value =
                serde_json::from_slice(line).map_err(|_| "journal_corrupt")?;
            if frame["schema_version"] != 1 {
                return Err("journal_schema_unsupported");
            }
            let payload = &frame["payload"];
            let digest =
                crate::auth::digest(&serde_json::to_vec(payload).map_err(|_| "journal_corrupt")?);
            if frame["digest"].as_str() != Some(&digest) {
                return Err("journal_corrupt");
            }
            records.push(payload.clone());
        }
        // Incomplete trailing frame was never acknowledged or dispatched.
        file.set_len(valid as u64)
            .map_err(|_| "storage_unavailable")?;
        file.seek(SeekFrom::End(0))
            .map_err(|_| "storage_unavailable")?;
        Ok((
            Self {
                file,
                path,
                bytes: valid as u64,
                failed: false,
            },
            records,
        ))
    }

    pub fn append(&mut self, payload: serde_json::Value) -> Result<(), &'static str> {
        if self.failed {
            return Err("storage_unavailable");
        }
        let digest =
            crate::auth::digest(&serde_json::to_vec(&payload).map_err(|_| "invalid_record")?);
        let mut bytes = serde_json::to_vec(
            &serde_json::json!({"schema_version":1,"digest":digest,"payload":payload}),
        )
        .map_err(|_| "invalid_record")?;
        bytes.push(b'\n');
        if self.bytes + bytes.len() as u64 > 64 * 1024 * 1024 {
            return Err("storage_full");
        }
        if self
            .file
            .seek(SeekFrom::End(0))
            .and_then(|_| self.file.write_all(&bytes))
            .and_then(|_| self.file.sync_all())
            .is_err()
        {
            self.failed = true;
            return Err("storage_unavailable");
        }
        self.bytes += bytes.len() as u64;
        Ok(())
    }

    pub fn compact(&mut self, records: Vec<serde_json::Value>) -> Result<(), &'static str> {
        if self.failed {
            return Err("storage_unavailable");
        }
        let temp = self
            .path
            .with_extension(format!("{}.tmp", crate::auth::random_id()));
        let mut options = OpenOptions::new();
        options.create_new(true).read(true).write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(4);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&temp).map_err(|_| "storage_unavailable")?;
        let mut replacement = Self {
            file,
            path: self.path.clone(),
            bytes: 0,
            failed: false,
        };
        let result = (|| {
            for record in records {
                replacement.append(record)?;
            }
            replacement
                .file
                .sync_all()
                .map_err(|_| "storage_unavailable")?;
            fs::rename(&temp, &self.path).map_err(|_| "storage_unavailable")?;
            Ok::<(), &'static str>(())
        })();
        match result {
            Ok(()) => {
                *self = replacement;
                Ok(())
            }
            Err(error) => {
                drop(replacement);
                let _ = fs::remove_file(temp);
                Err(error)
            }
        }
    }
}

fn is_link(path: &Path) -> Result<bool, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "storage_unavailable")?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Ok(metadata.file_attributes() & 0x400 != 0)
    }
    #[cfg(not(windows))]
    {
        Ok(metadata.file_type().is_symlink())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn failed_io_poisons_writer_and_torn_tail_recovers_only_synced_frames() {
        let root =
            std::env::temp_dir().join(format!("alexia-storage-{}", crate::auth::random_id()));
        let (mut journal, _) = Journal::open(&root, "test.jsonl").unwrap();
        journal.append(json!({"record":1})).unwrap();
        fs::write(root.join("readonly"), b"fixture").unwrap();
        let original = std::mem::replace(
            &mut journal.file,
            File::open(root.join("readonly")).unwrap(),
        );
        assert!(journal.append(json!({"record":2})).is_err());
        journal.file = original;
        assert!(journal.append(json!({"record":3})).is_err());
        drop(journal);
        OpenOptions::new()
            .append(true)
            .open(root.join("test.jsonl"))
            .unwrap()
            .write_all(b"{\"incomplete\":")
            .unwrap();
        let (journal, records) = Journal::open(&root, "test.jsonl").unwrap();
        assert_eq!(records, vec![json!({"record":1})]);
        drop(journal);
    }
}
