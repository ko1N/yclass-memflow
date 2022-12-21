use memflow::prelude::v1::*;
use parking_lot::RwLock;

pub struct Process {
    inner: RwLock<IntoProcessInstanceArcBox<'static>>,
}

impl Process {
    pub fn attach(os: OsInstanceArcBox<'static>, pid: u32) -> Result<Self> {
        // TODO: maps
        let inner = os.into_process_by_pid(pid)?;
        Ok(Self {
            inner: RwLock::new(inner),
        })
    }

    pub fn read(&self, address: usize, buf: &mut [u8]) {
        self.inner.write().read_into(address.into(), buf).ok();
    }

    pub fn write(&self, address: usize, buf: &[u8]) {
        self.inner.write().write(address.into(), buf).ok();
    }

    pub fn id(&self) -> u32 {
        use memflow::os::Process;
        self.inner.read().info().pid
    }

    pub fn can_read(&self, _address: usize) -> bool {
        // TODO: validate with mem maps/memflow api?
        true
    }

    pub fn name(&self) -> Result<String> {
        use memflow::os::Process;
        Ok(self.inner.read().info().name.to_string())
    }
}
