//! Local virtual machine state.
//!
//! # Configuration
//!
//! Key | Values | Default
//! ----|--------|--------
//! uri | URI of the hypervisor | qemu:///system
//! `interval` | Update interval, in seconds. | `5`
//! `format` | A string to customise the output of this block. See below for available placeholders. | `" $icon $running.eng(w:1) "`
//! `uru` | The path to the virtualization domain.
//!
//! Key       | Value                                  | Type   | Unit
//! ----------|----------------------------------------|--------|-----
//! `icon`    | A static icon                          | Icon   | -
//! `total`   | Total containers on the host           | Number | -
//! `running` | Virtual machines running on the host   | Number | -
//! `stopped` | Virtual machines stopped on the host   | Number | -
//! `paused`  | Virtual machines paused on the host    | Number | -
//! `total`   | Total Virtual machines on the host     | Number | -
//! `memory   | Total memory used by the virtual machines | Number | 
//! `cpu`     | Total percentage cpu used by the virtual machines | Number |
//!
//! # Example
//!
//! ```toml
//! [[block]]
//! block = "virt"
//! uri = "qemu:///system"
//! interval = 2
//! format = " $icon $running/$total ($memory@$cpu) "
//! ```
//!

use super::prelude::*;
use virt::connect::Connect;
use virt::error::Error;
use virt::sys;

#[derive(Deserialize, Debug, SmartDefault)]
#[serde(default)]
pub struct Config {
    #[default("qemu:///system".into())]
    uri: ShellString,
    format: FormatConfig,
    #[default(5.into())]
    interval: Seconds,
}

pub async fn run(config: Config, mut api: CommonApi) -> Result<()> {
    let mut format = config.format.with_default(" $icon $running/$total")?;
    // TODO: alt config
    
    let mut widget = Widget::new().with_format(format.clone());

    let virt_con = match Connect::open(&uri) {
        Ok(con) => con,
        Err(e) => return Err(Error("Failed to connect to hypervisor".to_string())),
    };

    loop {
        let virt_flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        let virt_active_doms = match virt_con.active_domains() {
            Ok(virt_dom_active) => virt_dom_active,
            _ => "?".to_string(),
        };

        let virt_inactive_doms = match virt_con.num_of_defined_domains() {
            Ok(virt_dom_inactive) => virt_dom_inactive,
            _ => "?".to_string(),
        };

        if let Ok(virt_domains) = virt_con.list_all_domains(virt_flags) {
            let mut virt_sum_cpu = 0;
            let mut virt_sum_mem = 0;
            let mut virt_sum_paused = 0;

            for dom in virt_domains {
                if let Ok(virt_dom_info) = dom.get_info() {
                    virt_sum_cpu += virt_dom_info.nr_virt_cpu;
                    virt_sum_mem += virt_dom_info.memory;
                    if virt_dom_info.state == sys::VIR_DOMAIN_PAUSED {
                        virt_sum_paused += 1;
                    }
                }
            }

        }

        widget.set_values(&map!(
            "icon" => Value::Icon("".to_string()),
            "total" => Value::from_integer(virt_active_doms + virt_inactive_doms),
            "running" => Value::from_integer(virt_active_doms),
            "stopped" => Value::from_integer(virt_inactive_doms),
            "paused" => Value::from_integer(virt_sum_paused),
        ));

        api.set_widget(&widget).await?;
    }

}
