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

fn spawn_boxer() -> std::process::Child {
    let out = Command::new("bochs_build\\bochs.exe")
        .arg("-q")
        .arg("-f")
        .arg("bochsrc.bxrc")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to run bochs");
    out
}

fn main() -> std::io::Result<()> {
    std::env::set_current_dir("..").unwrap();

    // Spawn 2 boxers to fight eachother
    let mut boxer_a_proc = spawn_boxer();
    let mut boxer_b_proc = spawn_boxer();

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
    
    // Buffers for socket reads
    let mut buf_a = [0u8; 0x18];
    let mut buf_b = [0u8; 0x18];
    let mut entry = 0u64;

    // Read from both sockets forever, in lockstep
    while let (Ok(()), Ok(())) = (boxer_a.read_exact(&mut buf_a), 
                                  boxer_b.read_exact(&mut buf_b)) {
        #[repr(C)]
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        struct Entry {
            typ:   u64,
            ticks: u64,
            rip:   u64,
        }

        // Convert the byte arrays to a meaningful type
        let a = unsafe { *(buf_a.as_ptr() as *const Entry) };
        let b = unsafe { *(buf_b.as_ptr() as *const Entry) };

        // Make sure each instance is doing the same thing
        /*if a.rip != b.rip {
            print!("Mismatch on entry {}\n", entry);
            print!("A: {:#x?}\nB: {:#x?}\n", a, b);
            panic!("DIVERGENCE");
        }*/

        // Update entry counter
        entry += 1;

        if (entry & 0xffffff) == 0 {
            let elapsed = elapsed_from(&start_time);

            let mi = entry as f64 / 1000000.0;
            print!("Processed entry {:12.2} million instrs | {:10.4} mips\n",
                   mi, mi / elapsed);
        }
    }

    boxer_a_proc.wait().expect("Boxer wait failed");
    boxer_b_proc.wait().expect("Boxer wait failed");

    Ok(())
}

