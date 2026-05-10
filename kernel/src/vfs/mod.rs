use alloc::{
    collections::btree_map::{BTreeMap, Entry},
    sync::Arc,
    vec::Vec,
};
use ksync::SpinLock;
use vfs::{BlockDevice, File, Filesystem, VfsError, VfsResult};

static FILE_SYSTEMS: FileSystems = FileSystems {
    file_systems: SpinLock::new(BTreeMap::new()),
};

struct FileSystems {
    file_systems: SpinLock<BTreeMap<Vec<u8>, Arc<dyn Filesystem>>>,
}

pub enum SupportedFs {
    Vsfs,
}

pub fn mount<BD: BlockDevice + 'static + Send + Sync>(
    path: &[u8],
    fs_type: SupportedFs,
) -> VfsResult<()> {
    match fs_type {
        SupportedFs::Vsfs => {
            let fs = vsfs::initialize::<BD>()?;
            match FILE_SYSTEMS.file_systems.lock().entry(path.to_vec()) {
                Entry::Occupied(_) => return Err(VfsError::AlreadyMounted),
                Entry::Vacant(e) => {
                    e.insert(fs);
                }
            }
        }
    }

    Ok(())
}

pub fn open(path: &[u8]) -> VfsResult<File> {
    let mounts = FILE_SYSTEMS.file_systems.lock();
    let (mount_path, fs) = find_mount(&mounts, path).ok_or(VfsError::Fs)?;

    let relative_path = if mount_path == b"/" {
        path.strip_prefix(b"/").unwrap_or(path)
    } else {
        path.strip_prefix(mount_path)
            .unwrap_or(path)
            .strip_prefix(b"/")
            .unwrap_or(b"")
    }
    .to_vec();

    drop(mounts);

    let root = fs.root()?;
    root.open(&relative_path)
}

// TODO(aeryz): This is messed up, it's a huge burden to be needing to iterate
// through all the keys. We need to store a mount tree s.t. we can easily go the
// longest matching path.
//
// Basically this:
//            "/"
//           /   \
//       "/mnt" "/mnt2"
//      /     \        \
//  "/mnt/a" "/mnt/b" "/mnt2/b"
fn find_mount<'a>(
    mounts: &'a BTreeMap<Vec<u8>, Arc<dyn Filesystem>>,
    path: &[u8],
) -> Option<(&'a [u8], Arc<dyn Filesystem>)> {
    mounts
        .iter()
        .filter(|(mount_path, _)| is_mount_prefix(mount_path, path))
        .max_by_key(|(mount_path, _)| mount_path.len())
        .map(|(mount_path, fs)| (mount_path.as_slice(), fs.clone()))
}

fn is_mount_prefix(mount: &[u8], path: &[u8]) -> bool {
    if mount == b"/" {
        return path.starts_with(b"/");
    }

    path == mount || path.starts_with(mount) && path.get(mount.len()) == Some(&b'/')
}
