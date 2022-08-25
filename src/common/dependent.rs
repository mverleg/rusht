use ::std::process::ExitStatus as ProcStatus;
use ::std::rc::Rc;

use ::async_std::sync::Mutex;
use ::async_std::sync::MutexGuard;
use ::log::debug;
use ::smallvec::SmallVec;
use ::time::Duration;

use crate::common::{StdWriter, Task};

#[derive(Debug)]
pub struct Dependency {
    name: Rc<String>,
    lock: Rc<Mutex<()>>,
    timeout: Duration,
    //TODO @mverleg: how to timeout mutex? race two futures?
}

impl Dependency {
    pub fn new_unlimited(name: String, lock: Mutex<()>) -> Self {
        Dependency::new_timeout(Rc::new(name), Rc::new(lock), Duration::new(i64::MAX, 0))
    }

    pub fn new_timeout(name: Rc<String>, lock: Rc<Mutex<()>>, timeout: Duration) -> Self {
        Dependency {
            name,
            lock,
            timeout,
        }
    }
}

#[derive(Debug)]
pub struct Dependent {
    task: Task,
    name: Rc<String>,
    current: Rc<Mutex<()>>,
    dependencies: SmallVec<[Dependency; 1]>,
}

impl Dependent {
    pub fn new(task: Task, dependencies: impl Into<SmallVec<[Dependency; 1]>>) -> Self {
        let name = Rc::new(task.as_str());
        Dependent {
            task,
            name,
            current: Rc::new(Mutex::new(())),
            dependencies: dependencies.into(),
        }
    }

    pub fn depends_on(&mut self, other: &Dependent) {
        self.dependencies.push(Dependency {
            name: other.name.clone(),
            lock: other.current.clone(),
            timeout: Default::default()
        })
    }

    pub async fn await_and_exec(&self) -> ProcStatus {
        for dependency in &self.dependencies {
            let _guard: MutexGuard<()> = match dependency.lock.try_lock() {
                None => {
                    debug!("{} needs {} which needs to be awaited", self.name, dependency.name);
                    dependency.lock.lock().await
                }
                Some(guard) => {
                    debug!("{} needs {} which is immediately available", self.name, dependency.name);
                    guard
                },
            };
        };
        let status = self.task.execute_with_stdout(false, &mut StdWriter::stdout()).await;
        status
    }
}

pub async fn run_all(dependents: &[Dependent]) {
    dependents.iter()
        .map(|dep| dep.await_and_exec())
        .collect::<Vec<_>>();
    //join!(dependents).await
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use ::smallvec::smallvec;

    use super::*;

    #[test]
    fn dependency_tree() {
        let top = Dependent::new(Task::noop(), smallvec![]);
        let mut mid1 = Dependent::new(Task::noop(), smallvec![]);
        mid1.depends_on(&top);
        let mut mid2 = Dependent::new(Task::noop(), smallvec![]);
        mid2.depends_on(&top);
        let mut botm = Dependent::new(Task::noop(), smallvec![]);
        botm.depends_on(&mid1);
        botm.depends_on(&mid2);
        let deps = vec![botm, mid1, top, mid2];
        run_all(&deps);
    }
}
