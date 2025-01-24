use std::io::Write;

pub struct Clipboard {
    bin: String,
}

pub fn new(bin: impl Into<String>) -> Clipboard {
    Clipboard { bin: bin.into() }
}

impl Clipboard {
    pub fn copy(&self, content: Vec<u8>) {
        let mut child = std::process::Command::new(&self.bin)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Error executing clipboard");
        child
            .stdin
            .as_mut()
            .expect("Failed to open stdin")
            .write_all(&content)
            .unwrap();
        let status = child.wait().expect("Error executing clipboard");
        if !status.success() {
            panic!("Error executing clipboard");
        }
    }
}
