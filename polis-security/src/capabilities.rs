use polis_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    // File system capabilities
    Chown,
    DacOverride,
    DacReadSearch,
    Fowner,
    Fsetid,
    Kill,
    Setgid,
    Setuid,
    Setpcap,
    LinuxImmutable,
    NetBindService,
    NetBroadcast,
    NetAdmin,
    NetRaw,
    IpcLock,
    IpcOwner,
    SysModule,
    SysRawio,
    SysChroot,
    SysPtrace,
    SysPacct,
    SysAdmin,
    SysBoot,
    SysNice,
    SysResource,
    SysTime,
    SysTtyConfig,
    Mknod,
    Lease,
    AuditWrite,
    AuditControl,
    Setfcap,
    MacOverride,
    MacAdmin,
    Syslog,
    WakeAlarm,
    BlockSuspend,
    AuditRead,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitySet {
    pub effective: HashSet<Capability>,
    pub permitted: HashSet<Capability>,
    pub inheritable: HashSet<Capability>,
}

#[derive(Default)]
pub struct CapabilityManager {
    current_caps: CapabilitySet,
}

impl CapabilityManager {
    pub fn new() -> Self {
        Self {
            current_caps: CapabilitySet {
                effective: HashSet::new(),
                permitted: HashSet::new(),
                inheritable: HashSet::new(),
            },
        }
    }

    pub async fn drop_capabilities(&mut self, caps: Vec<Capability>) -> Result<()> {
        let caps_clone = caps.clone();
        for cap in caps {
            self.current_caps.effective.remove(&cap);
            self.current_caps.permitted.remove(&cap);
            self.current_caps.inheritable.remove(&cap);
        }

        println!("🔒 Capabilities removidas: {:?}", caps_clone);
        Ok(())
    }

    pub async fn add_capabilities(&mut self, caps: Vec<Capability>) -> Result<()> {
        let caps_clone = caps.clone();
        for cap in caps {
            self.current_caps.effective.insert(cap.clone());
            self.current_caps.permitted.insert(cap.clone());
            self.current_caps.inheritable.insert(cap);
        }

        println!("🔓 Capabilities adicionadas: {:?}", caps_clone);
        Ok(())
    }

    pub async fn set_capabilities(&mut self, caps: CapabilitySet) -> Result<()> {
        self.current_caps = caps;
        println!("🔧 Capabilities definidas");
        Ok(())
    }

    pub async fn get_current_capabilities(&self) -> Result<CapabilitySet> {
        Ok(self.current_caps.clone())
    }

    pub async fn create_minimal_capset(&mut self) -> Result<()> {
        // Minimal set of capabilities for basic container operations
        let minimal_caps = [
            Capability::Chown,
            Capability::DacOverride,
            Capability::Fowner,
            Capability::Fsetid,
            Capability::Kill,
            Capability::Setgid,
            Capability::Setuid,
            Capability::Setpcap,
            Capability::NetBindService,
            Capability::NetRaw,
            Capability::IpcLock,
            Capability::SysChroot,
            Capability::AuditWrite,
            Capability::Setfcap,
        ];

        self.current_caps = CapabilitySet {
            effective: minimal_caps.iter().cloned().collect(),
            permitted: minimal_caps.iter().cloned().collect(),
            inheritable: HashSet::new(),
        };

        println!("🔒 Conjunto mínimo de capabilities definido");
        Ok(())
    }

    pub async fn create_privileged_capset(&mut self) -> Result<()> {
        // All capabilities (privileged mode)
        let all_caps = [
            Capability::Chown,
            Capability::DacOverride,
            Capability::DacReadSearch,
            Capability::Fowner,
            Capability::Fsetid,
            Capability::Kill,
            Capability::Setgid,
            Capability::Setuid,
            Capability::Setpcap,
            Capability::LinuxImmutable,
            Capability::NetBindService,
            Capability::NetBroadcast,
            Capability::NetAdmin,
            Capability::NetRaw,
            Capability::IpcLock,
            Capability::IpcOwner,
            Capability::SysModule,
            Capability::SysRawio,
            Capability::SysChroot,
            Capability::SysPtrace,
            Capability::SysPacct,
            Capability::SysAdmin,
            Capability::SysBoot,
            Capability::SysNice,
            Capability::SysResource,
            Capability::SysTime,
            Capability::SysTtyConfig,
            Capability::Mknod,
            Capability::Lease,
            Capability::AuditWrite,
            Capability::AuditControl,
            Capability::Setfcap,
            Capability::MacOverride,
            Capability::MacAdmin,
            Capability::Syslog,
            Capability::WakeAlarm,
            Capability::BlockSuspend,
            Capability::AuditRead,
        ];

        self.current_caps = CapabilitySet {
            effective: all_caps.iter().cloned().collect(),
            permitted: all_caps.iter().cloned().collect(),
            inheritable: all_caps.iter().cloned().collect(),
        };

        println!("🔓 Conjunto privilegiado de capabilities definido");
        Ok(())
    }

    pub async fn list_capabilities(&self) -> Result<Vec<Capability>> {
        Ok(self.current_caps.effective.iter().cloned().collect())
    }
}
