pub trait SecretStore: Send + Sync {
    fn put(&self, reference: &str, value: &str) -> Result<(), &'static str>;
    fn get(&self, reference: &str) -> Result<String, &'static str>;
    fn delete(&self, reference: &str) -> Result<(), &'static str>;
}
pub struct OsSecretStore;

fn target(reference: &str) -> Result<Vec<u16>, &'static str> {
    if reference.is_empty()
        || reference.len() > 256
        || !reference
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("invalid_secret_reference");
    }
    Ok(format!("AlexIA/integrations/{reference}\0")
        .encode_utf16()
        .collect())
}

#[cfg(windows)]
impl SecretStore for OsSecretStore {
    fn put(&self, reference: &str, value: &str) -> Result<(), &'static str> {
        use windows_sys::Win32::Security::Credentials::*;
        let mut name = target(reference)?;
        let mut bytes = value.as_bytes().to_vec();
        if bytes.is_empty() || bytes.len() > 2400 {
            return Err("invalid_secret");
        }
        let credential = CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: name.as_mut_ptr(),
            CredentialBlobSize: bytes.len() as u32,
            CredentialBlob: bytes.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            ..Default::default()
        };
        // SAFETY: all buffers remain live for the synchronous native call; Windows copies the blob.
        let success = unsafe { CredWriteW(&credential, 0) };
        bytes.fill(0);
        if success == 0 {
            Err("vault_unavailable")
        } else {
            Ok(())
        }
    }
    fn get(&self, reference: &str) -> Result<String, &'static str> {
        use windows_sys::Win32::Security::Credentials::*;
        let name = target(reference)?;
        let mut credential = std::ptr::null_mut();
        // SAFETY: CredRead allocates the structure until CredFree; read only the declared blob.
        unsafe {
            if CredReadW(name.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) == 0 {
                return Err("vault_unavailable");
            }
            let size = (*credential).CredentialBlobSize as usize;
            let value = if size == 0 || size > 2400 || (*credential).CredentialBlob.is_null() {
                Err("invalid_secret")
            } else {
                String::from_utf8(
                    std::slice::from_raw_parts((*credential).CredentialBlob, size).to_vec(),
                )
                .map_err(|_| "invalid_secret")
            };
            CredFree(credential.cast());
            value
        }
    }
    fn delete(&self, reference: &str) -> Result<(), &'static str> {
        use windows_sys::Win32::{
            Foundation::{ERROR_NOT_FOUND, GetLastError},
            Security::Credentials::*,
        };
        let name = target(reference)?;
        // SAFETY: target is NUL terminated and contains only an internal reference.
        if unsafe { CredDeleteW(name.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0
            && unsafe { GetLastError() } != ERROR_NOT_FOUND
        {
            return Err("vault_unavailable");
        }
        Ok(())
    }
}

#[cfg(not(windows))]
impl SecretStore for OsSecretStore {
    fn put(&self, reference: &str, _: &str) -> Result<(), &'static str> {
        target(reference)?;
        Err("vault_unavailable_on_this_platform")
    }
    fn get(&self, reference: &str) -> Result<String, &'static str> {
        target(reference)?;
        Err("vault_unavailable_on_this_platform")
    }
    fn delete(&self, reference: &str) -> Result<(), &'static str> {
        target(reference)?;
        Err("vault_unavailable_on_this_platform")
    }
}
