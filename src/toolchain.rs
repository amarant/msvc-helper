use std::ffi::OsString;
use std::path::PathBuf;
use setup_config::{SetupConfiguration, SetupInstance};
use std::cmp::Ordering;

#[derive(Debug)]
pub struct VisualStudioInstallationInstance {
    instance_id: String,
    installation_name: String,
    installation_path: PathBuf,
    installation_version: String,
    product_path: PathBuf,
    platform_toolset: String,
}

impl VisualStudioInstallationInstance {
    fn set_platform_toolset(&mut self, platform_toolset: String) {
        self.platform_toolset = platform_toolset;
    }
}

fn os_to_res_string(res: Result<OsString, i32>) -> Result<String, Option<i32>> {
    match res {
        Ok(s) => match s.into_string() {
            Ok(s) => Ok(s),
            Err(_) => Err(None),
        },
        Err(i) => Err(Some(i)),
    }
}

fn os_to_res_pathbuf(res: Result<OsString, i32>) -> Result<PathBuf, Option<i32>> {
    match res {
        Ok(s) => match s.into_string() {
            Ok(s) => Ok(PathBuf::from(s)),
            Err(_) => Err(None),
        },
        Err(i) => Err(Some(i)),
    }
}

fn transform(instance: SetupInstance) -> Result<VisualStudioInstallationInstance, Option<i32>> {
    let instance_id = os_to_res_string(instance.instance_id())?;
    let installation_name = os_to_res_string(instance.installation_name())?;
    let installation_path = os_to_res_pathbuf(instance.installation_path())?;
    let installation_version = os_to_res_string(instance.installation_version())?;
    let product_path = os_to_res_pathbuf(instance.product_path())?;
    Ok(VisualStudioInstallationInstance {
        instance_id,
        installation_name,
        installation_path,
        installation_version,
        product_path,
        platform_toolset: "".into(),
    })
}

pub fn get_toolchains() -> Vec<VisualStudioInstallationInstance> {
    let config = SetupConfiguration::new().unwrap();
    let iter = config.enum_all_instances().unwrap();
    let mut toolchains: Vec<VisualStudioInstallationInstance> = Vec::new();
    for instance in iter {
        let mut instance: VisualStudioInstallationInstance = match instance {
            Ok(instance) => match transform(instance) {
                Ok(instance) => instance,
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        let major_version = &instance.installation_version.clone()[..2];
        match &major_version as &str {
            "15" => {
                instance.set_platform_toolset("v141".into());
                toolchains.push(instance);
            }
            "14" => {
                instance.set_platform_toolset("v140".into());
                toolchains.push(instance);
            }
            _ => (),
        }
    }
    toolchains.sort_by(|a, b| {
        a.installation_version
            .split(".")
            .zip(b.installation_version.split("."))
            .map(|(j, i)| {
                let len_cmp = i.len().cmp(&j.len());
                if len_cmp != Ordering::Equal {
                    return len_cmp;
                }
                i.cmp(j)
            })
            .filter(|c| c != &Ordering::Equal)
            .next()
            .unwrap_or(Ordering::Equal)
    });
    toolchains
}

pub fn get_lasted_platform_toolset() -> Option<String> {
    get_toolchains()
        .iter()
        .next()
        .map(|v| v.platform_toolset.clone())
}
