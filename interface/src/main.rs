use std::{
    fs::OpenOptions,
    io::Write,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use defmt_decoder::{DecodeError, Table};
use probe_rs::{Session, SessionConfig};

fn main() -> Result<()> {
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open("out.csv")?;

    let elf_path = "../embeded/target/thumbv6m-none-eabi/release/embeded";
    let elf_bytes = std::fs::read(elf_path)?;
    let table = Table::parse(&elf_bytes)?.unwrap();

    let mut session = Session::auto_attach("STM32C011F4", SessionConfig::default())?;
    let mut core = session.core(0)?;
    core.reset()?;

    let mut rtt = probe_rs::rtt::Rtt::attach(&mut core)?;

    let up_channel = rtt
        .up_channel(0)
        .ok_or_else(|| anyhow::anyhow!("RTT up channel 0 not found"))?;

    let mut stream_decoder = table.new_stream_decoder();
    let mut buf = [0u8; 1024];
    loop {
        let count = up_channel.read(&mut core, &mut buf[..])?;

        if count > 0 {
            stream_decoder.received(&buf[..count]);

            loop {
                match stream_decoder.decode() {
                    Ok(frame) => {
                        let time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        let frame = frame.display(false).to_string();
                        let line = format!("{time},{}\n", frame.split_once(' ').unwrap().1);
                        print!("{line}");
                        output.write_all(&line.as_bytes())?;
                    }
                    Err(DecodeError::UnexpectedEof) => break,
                    Err(DecodeError::Malformed) => continue,
                }
            }
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}
