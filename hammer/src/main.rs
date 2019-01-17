use std::process::{Command, Stdio};
use std::io::BufReader;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

/// Get elapsed time in seconds
pub fn elapsed_from(start: &Instant) -> f64 {
    let dur = start.elapsed();
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1_000_000_000.0
}

fn spawn_boxer(id: usize, snapshot: bool) -> std::process::Child {
    if !snapshot {
        let out = Command::new("bochs_build\\bochs.exe")
            .arg("-q")
            .arg("-f")
            .arg("bochsrc.bxrc")
            .stderr(File::create(format!("logerr{}.txt", id)).unwrap())
            .stdout(File::create(format!("log{}.txt", id)).unwrap())
            .spawn()
            .expect("Failed to run bochs");
        std::thread::sleep_ms(2000);
        out
    } else {
        let out = Command::new("bochs_build\\bochs.exe")
            .arg("-r")
            .arg("apples")
            .stderr(File::create(format!("logerr{}.txt", id)).unwrap())
            .stdout(File::create(format!("log{}.txt", id)).unwrap())
            .spawn()
            .expect("Failed to run bochs");
        std::thread::sleep_ms(2000);
        out
    }
}

fn main() -> std::io::Result<()> {
    std::env::set_current_dir("..").unwrap();

    // Spawn 2 boxers to fight eachother
    let mut boxer_a_proc = spawn_boxer(1, false);
    let mut boxer_b_proc = spawn_boxer(2, false);

    {
        // Pick the one to take a snapshot
        File::create(format!("foopie{}", boxer_b_proc.id())).unwrap();
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));

    let boxer_a_pipename = format!("\\\\.\\pipe\\mynamedpipe{}", boxer_a_proc.id());
    let boxer_b_pipename = format!("\\\\.\\pipe\\mynamedpipe{}", boxer_b_proc.id());

    let mut boxer_a = None;
    while boxer_a.is_none() {
        if let Ok(fd) = File::open(&boxer_a_pipename) { boxer_a = Some(fd); }
    }
    let boxer_a = boxer_a.unwrap();

    let mut boxer_b = None;
    while boxer_b.is_none() {
        if let Ok(fd) = File::open(&boxer_b_pipename) { boxer_b = Some(fd); }
    }
    let boxer_b = boxer_b.unwrap();

    // Create buffered readers around them to do things in bulk
    let mut boxer_a = BufReader::with_capacity(16 * 1024 * 1024, boxer_a);
    let mut boxer_b = BufReader::with_capacity(16 * 1024 * 1024, boxer_b);

    print!("OPENED PIPES\n");

    let start_time = Instant::now();
    
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    struct Entry {
        typ:     u64,
        arg:     u64,
        ticks:   u64,
        tilnext: u64,
        rip:     u64,
        icount:  u64,
    }

    // Buffers for socket reads
    let mut buf_a = [0u8; std::mem::size_of::<Entry>()];
    let mut buf_b = [0u8; std::mem::size_of::<Entry>()];
    let mut entry = 0u64;

    for iter in 0..2 {
        // Read from both sockets forever, in lockstep
        loop {
            if boxer_b.read_exact(&mut buf_b).is_err() { break; }
            let b = unsafe { *(buf_b.as_ptr() as *const Entry) };

            // Special case for snapshot, break before we consume anything from
            // boxer_a
            if b.typ == 0x13371337 {
                break;
            }

            if boxer_a.read_exact(&mut buf_a).is_err() { break; }
            let a = unsafe { *(buf_a.as_ptr() as *const Entry) };

            // Make sure each instance is doing the same thing
            if a != b {
                print!("Mismatch on entry {}\n", entry);
                print!("A: {:#x?}\nB: {:#x?}\n", a, b);

                std::thread::sleep_ms(500);

                panic!("DIVERGENCE");
            }

            // Update entry counter
            entry += 1;

            if (entry & 0xffffff) == 0 {
                let elapsed = elapsed_from(&start_time);

                let mi = entry as f64 / 1000000.0;
                print!("Processed entry {:12.2} million instrs | {:10.4} mips\n",
                       mi, mi / elapsed);
            }
        }

        print!("Exited loop on entry {}\n", entry);
        boxer_b_proc.kill().expect("Boxer b kill failed");
        print!("Boxer b done\n");
        boxer_b_proc = spawn_boxer(3, true);
        print!("Respawned boxer b as snapshot\n");

        // Reopen reader
        let mut boxer_b_reopen = None;
        let boxer_b_pipename = format!("\\\\.\\pipe\\mynamedpipe{}", boxer_b_proc.id());
        while boxer_b_reopen.is_none() {
            if let Ok(fd) = File::open(&boxer_b_pipename) { boxer_b_reopen = Some(fd); }
        }
        let boxer_b_reopen = boxer_b_reopen.unwrap();
        boxer_b = BufReader::with_capacity(16 * 1024 * 1024, boxer_b_reopen);

        print!("Boxer b back up!\n");
    }

    boxer_a_proc.wait().expect("Boxer wait failed");
    boxer_b_proc.wait().expect("Boxer wait failed");
    print!("All boxers down\n");

    Ok(())
}

