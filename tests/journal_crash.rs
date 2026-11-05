use alexia::{
    auth::{now, random_id},
    jobs::{JobManager, JobRequest, Status},
};
use serde_json::json;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
fn journal_child_helper() {
    let Some(root) = std::env::var_os("ALEXIA_CRASH_FIXTURE_ROOT") else {
        return;
    };
    let mut jobs = JobManager::open(std::path::Path::new(&root)).unwrap();
    let job = jobs
        .enqueue("a", JobRequest::fixture("one-effect", json!({})), now())
        .unwrap();
    jobs.record_effect(&job.id, "synthetic-remote-effect", now())
        .unwrap();
    let mut socket =
        TcpStream::connect(std::env::var("ALEXIA_CRASH_FIXTURE_ADDRESS").unwrap()).unwrap();
    socket.write_all(b"effect-sent").unwrap();
    // The parent terminates this process after delivery, before any receipt exists.
    loop {
        std::thread::park();
    }
}

#[test]
fn process_killed_after_dispatch_recovers_without_replaying_the_effect() {
    let root = std::env::temp_dir().join(format!("alexia-crash-{}", random_id()));
    fs::create_dir_all(&root).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "journal_child_helper", "--nocapture"])
        .env("ALEXIA_CRASH_FIXTURE_ROOT", &root)
        .env(
            "ALEXIA_CRASH_FIXTURE_ADDRESS",
            listener.local_addr().unwrap().to_string(),
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut observed = false;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((mut socket, _)) => {
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut bytes = [0; 11];
                socket.read_exact(&mut bytes).unwrap();
                assert_eq!(&bytes, b"effect-sent");
                observed = true;
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(error) => panic!("{error}"),
        }
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(observed, "child did not dispatch the fixture effect");
    let jobs = JobManager::open(&root).unwrap();
    assert_eq!(jobs.pending_effects().len(), 1);
    assert_eq!(jobs.list("a")[0].status, Status::Reconciling);
    assert!(matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock));
}
