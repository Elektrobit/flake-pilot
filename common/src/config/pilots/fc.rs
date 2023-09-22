use serde::Deserialize;
use serde_yaml::Value;

/// Additional segment of Firecracker configuration for parameters
#[derive(Debug, Deserialize, PartialEq)]
pub struct FirecrackerRuntimeParams {
    boot_args: Option<Vec<String>>,
    mem_size_mib: Option<u32>,
    vcpu_count: Option<u8>,
    cache_type: Option<String>,
    overlay_size: Option<String>,
    rootfs_image_path: String,
    kernel_image_path: String,
    initrd_path: String,
}

impl FirecrackerRuntimeParams {
    pub fn boot_args(&self) -> Option<&Vec<String>> {
        self.boot_args.as_ref()
    }

    pub fn mem_size_mib(&self) -> Option<u32> {
        self.mem_size_mib
    }

    pub fn vcpu_count(&self) -> Option<u8> {
        self.vcpu_count
    }

    pub fn cache_type(&self) -> Option<&String> {
        self.cache_type.as_ref()
    }

    pub fn overlay_size(&self) -> Option<&String> {
        self.overlay_size.as_ref()
    }

    pub fn rootfs_image_path(&self) -> &str {
        self.rootfs_image_path.as_ref()
    }

    pub fn kernel_image_path(&self) -> &str {
        self.kernel_image_path.as_ref()
    }

    pub fn initrd_path(&self) -> &str {
        self.initrd_path.as_ref()
    }
}

impl From<Value> for FirecrackerRuntimeParams {
    fn from(value: Value) -> Self {
        match serde_yaml::from_value::<FirecrackerRuntimeParams>(value) {
            Ok(params) => params,
            Err(_) => FirecrackerRuntimeParams {
                boot_args: None,
                mem_size_mib: None,
                vcpu_count: None,
                cache_type: None,
                overlay_size: None,
                rootfs_image_path: "".to_string(),
                kernel_image_path: "".to_string(),
                initrd_path: "".to_string(),
            },
        }
    }
}
