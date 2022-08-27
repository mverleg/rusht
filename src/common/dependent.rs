use ::std::process::ExitStatus as ProcStatus;
use ::std::rc::Rc;

use ::futures::future::join_all;
use ::log::debug;
use ::smallvec::SmallVec;
use smallvec::smallvec;

use crate::common::{StdWriter, Task};
use crate::common::async_gate::AsyncGate;

#[derive(Debug)]
pub struct Dependency {
    name: Rc<String>,
    gate: AsyncGate,
}

impl Dependency {
    #[allow(dead_code)]
    pub fn new_with_gate(name: Rc<String>, gate: AsyncGate) -> Self {
        Dependency {
            name,
            gate,
        }
    }
}

#[derive(Debug)]
pub struct Dependent {
    task: Task,
    name: Rc<String>,
    current: AsyncGate,
    dependencies: SmallVec<[Dependency; 1]>,
}

impl Dependent {
    pub fn new_named(name: impl Into<String>, task: Task) -> Self {
        Dependent {
            task,
            name: Rc::new(name.into()),
            current: AsyncGate::new(),
            dependencies: smallvec![],
        }
    }

    pub fn new(task: Task) -> Self {
        let name = task.as_str();
        Dependent::new_named(name, task)
    }

    pub fn depends_on(&mut self, other: &Dependent) {
        self.dependencies.push(Dependency {
            name: other.name.clone(),
            gate: other.current.clone(),
        })
    }

    pub async fn await_and_exec(&self) -> ProcStatus {
        let count = self.dependencies.len();
        for (nr, dependency) in self.dependencies.iter().enumerate() {
            if dependency.gate.is_open() {
                debug!("{} needs {} [{}/{}] which is immediately available", self.name, dependency.name, nr + 1, count);
            } else {
                debug!("{} needs {} [{}/{}] which needs to be awaited", self.name, dependency.name, nr + 1, count);
                let _: () = dependency.gate.wait().await;
                debug!("{} was waiting for {} [{}/{}] which just became available", self.name, dependency.name, nr + 1, count);
            }
        };
        let status = self.task.execute_with_stdout(false, &mut StdWriter::stdout()).await;
        self.current.open();
        status
    }
}

#[cfg(test)]
mod tests {
    use ::std::time::Duration;

    use ::async_std::task::sleep;
    use ::futures::future::Either;
    use ::futures::future::select;

    use super::*;

    #[async_std::test]
    async fn dependency_tree() {
        let top = Dependent::new_named("top", Task::noop());
        let mut mid1 = Dependent::new_named("mid1", Task::noop());
        mid1.depends_on(&top);
        let mut mid2 = Dependent::new_named("mid2", Task::noop());
        mid2.depends_on(&top);
        let mut botm = Dependent::new_named("bottom", Task::noop());
        botm.depends_on(&mid1);
        botm.depends_on(&mid2);
        let deps = vec![botm, mid1, top, mid2];
        match select(
                Box::pin(sleep(Duration::from_secs(3))),
                Box::pin(run_all(deps))
        ).await {
            Either::Left(_) => panic!("timeout"),
            Either::Right(_) => {}
        }
    }
}
