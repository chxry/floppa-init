use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use nix::unistd::sethostname;
use nix::mount::{mount, MsFlags};

const NAME: &str = env!("CARGO_BIN_NAME");
const VER: &str = env!("CARGO_PKG_VERSION");

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
  if process::id() != 1 {
    println!("{} must be ran as pid 1", NAME);
    return;
  }
  println!("{} v{}", NAME, VER);
  println!("{:?}", hostname());
  println!("{:?}", mounts());
  loop {
    Command::new("getty")
      .args(["38400", "tty1"])
      .status()
      .unwrap();
  }
}

fn hostname() -> Result {
  match fs::read_to_string("/etc/hostname") {
    Ok(s) => match sethostname(s.trim()) {
      Ok(_) => Ok(()),
      Err(e) => Err(format!("couldn't set hostname: {}", e).into()),
    },
    Err(e) => Err(format!("couldn't open /etc/hostname: {}", e).into()),
  }
}

#[derive(Debug)]
struct Mount<'a> {
  spec: &'a str,
  file: &'a Path,
  vfs_type: &'a str,
  opts: Vec<&'a str>,
}

impl<'a> Mount<'a> {
  fn parse(line: &'a str) -> Result<Self> {
    let mut split = line.split_whitespace();
    let spec = split.next().ok_or("expected fs_spec")?;
    let file = Path::new(split.next().ok_or("expected fs_file")?);
    let vfs_type = split.next().ok_or("expected fs_vfstype")?;
    let opts = split
      .next()
      .ok_or("expected fs_mntops")?
      .split(',')
      .collect();
    Ok(Self {
      spec,
      file,
      vfs_type,
      opts,
    })
  }
}

fn get_mounts(file: &str) -> Vec<Mount<'_>> {
  let mut mounts = vec![];
  for l in file.lines() {
    if l.starts_with('#') || l.is_empty() {
      continue;
    }
    match Mount::parse(l) {
      Ok(m) => mounts.push(m),
      Err(e) => println!("couldn't parse fstab entry: {}", e),
    };
  }
  mounts
}

fn mounts() -> Result {
  let mount_opts = HashMap::from([
    ("nosuid", MsFlags::MS_NOSUID),
    ("nodev", MsFlags::MS_NODEV),
    ("noexec", MsFlags::MS_NOEXEC),
    ("sync", MsFlags::MS_SYNCHRONOUS),
    ("dirsync", MsFlags::MS_DIRSYNC),
    ("noatime", MsFlags::MS_NOATIME),
    ("nodiratime", MsFlags::MS_NODIRATIME),
    ("mand", MsFlags::MS_MANDLOCK),
    ("relatime", MsFlags::MS_RELATIME),
    ("strictatime", MsFlags::MS_STRICTATIME),
    ("rbind", MsFlags::MS_BIND | MsFlags::MS_REC),
  ]);

  let mounted = fs::read_to_string("/proc/mounts")?;
  let mounted = get_mounts(&mounted);
  for m in get_mounts(&fs::read_to_string("/etc/fstab")?) {
    if m.opts.iter().any(|o| *o == "noauto") || m.vfs_type == "swap" {
      continue;
    }
    let file = if let Some(uuid) = m.spec.strip_prefix("UUID=") {
      fs::canonicalize(format!("/dev/disk/by-uuid/{}", uuid))?
    } else {
      PathBuf::from(m.spec)
    };
    let remount = mounted.iter().any(|e| e.file == m.file);
    let mut flags = if remount {
      MsFlags::MS_REMOUNT
    } else {
      MsFlags::empty()
    };
    let opts: Vec<_> = m
      .opts
      .into_iter()
      .filter(|o| mount_opts.get(*o).map(|f| flags |= *f).is_none())
      .collect();
    match mount(
      Some(&file),
      m.file,
      Some(m.vfs_type),
      flags,
      Some(opts.join(",").as_str()),
    ) {
      Ok(_) => println!(
        "{} {:?}",
        if remount { "remounted" } else { "mounted" },
        m.file
      ),
      Err(e) => println!("couldnt mount {:?}: {}", m.file, e),
    }
  }
  Ok(())
}
