use simple;
use std::io::Read;
use std::process::{Command, Stdio};
// use std::thread;
// use std::time::Duration;

fn capture_frames(w: usize, h: usize) -> std::io::Result<()> {
    let mut app = simple::Window::new("Olego app", 720, 1440);
    app.set_color(0, 0, 0, 255);
    let frame_size = 4 * w * h;
    let mut adb = Command::new("adb")
        .args(&["exec-out", "screenrecord", "--output-format=h264", "-"])
        .stdout(Stdio::piped())
        .spawn()?;
    let mut ffmpeg = Command::new("ffmpeg")
        .args(&[
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "h264",
            "-i",
            "pipe:0",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-vsync",
            "0",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let mut adb_out = adb.stdout.take().expect("adb stdout");
        let mut ff_in = ffmpeg.stdin.take().expect("ffmpeg stdin");
        std::thread::spawn(move || {
            std::io::copy(&mut adb_out, &mut ff_in).ok();
            drop(ff_in);
        });
    }
    let mut ff_out = ffmpeg.stdout.take().expect("ffmpeg stdout");
    let mut buf = vec![0u8; frame_size];
    while app.next_frame() {
        app.clear();
        let mut read = 0;
        while read < frame_size {
            match ff_out.read(&mut buf[read..]) {
                Ok(0) => {
                    let _ = ffmpeg.kill();
                    let _ = adb.kill();
                    return Ok(());
                }
                Ok(n) => read += n,
                Err(e) => {
                    eprintln!("read error: {}", e);
                    let _ = ffmpeg.kill();
                    let _ = adb.kill();
                    return Err(e);
                }
            }
        }
        let frame = buf.clone();

        let mut frame = app.load_image(&frame).unwrap();
        app.draw_image(&mut frame, 0, 0);
        // if i % 30 == 0 {
        //     let filename = format!("frame_{:04}.rgba", i);
        //     std::fs::write(&filename, &frame)?;
        //     println!("Wrote {}", filename);
        // }
        // thread::sleep(Duration::from_millis(1000 / 24));
    }
    let _ = ffmpeg.kill();
    let _ = adb.kill();
    Ok(())
}

fn main() -> std::io::Result<()> {
    let w = 720usize;
    let h = 1440usize;
    capture_frames(w, h)
}
