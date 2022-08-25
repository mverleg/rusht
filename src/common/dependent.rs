use ::std::process::ExitStatus as ProcStatus;
use ::std::rc::Rc;

use ::futures::future::join_all;
use ::log::debug;
use ::log::warn;
use ::smallvec::SmallVec;
use ::wait_for_me::CountDownLatch;

use crate::common::{StdWriter, Task};

pub struct Dependency {
    name: Rc<String>,
    gate: Rc<CountDownLatch>,
}

impl Dependency {
    pub fn new_with_gate(name: Rc<String>, gate: Rc<CountDownLatch>) -> Self {
        Dependency {
            name,
            gate,
        }
    }
}

pub struct Dependent {
    task: Task,
    name: Rc<String>,
    current: Rc<CountDownLatch>,
    dependencies: SmallVec<[Dependency; 1]>,
}

impl Dependent {
    pub fn new(task: Task, dependencies: impl Into<SmallVec<[Dependency; 1]>>) -> Self {
        let name = Rc::new(task.as_str());
        Dependent {
            task,
            name,
            current: Rc::new(CountDownLatch::new(1)),
            dependencies: dependencies.into(),
        }
    }

    pub fn depends_on(&mut self, other: &Dependent) {
        self.dependencies.push(Dependency {
            name: other.name.clone(),
            gate: other.current.clone(),
        })
    }

    pub async fn await_and_exec(&self) -> ProcStatus {
        eprintln!(">> {} depends on {}", &self.name, self.dependencies.len());  //TODO @mverleg: TEMPORARY! REMOVE THIS!
        let count = self.dependencies.len();
        for (nr, dependency) in self.dependencies.iter().enumerate() {
            if dependency.gate.count().await == 0 {
                debug!("{} needs {} [{}/{}] which is immediately available", self.name, dependency.name, nr + 1, count);
            } else {
                debug!("{} needs {} [{}/{}] which needs to be awaited", self.name, dependency.name, nr + 1, count);
                let _: () = dependency.gate.wait().await;
                debug!("{} was waiting for {} [{}/{}] which just became available", self.name, dependency.name, nr + 1, count);
            }
        };
        let status = self.task.execute_with_stdout(false, &mut StdWriter::stdout()).await;
        self.current.count_down().await;
        status
    }
}

pub async fn run_all(dependents: Vec<Dependent>) -> ProcStatus {
    join_all(dependents.iter()
        .map(|dep| dep.await_and_exec())
        .collect::<Vec<_>>())
        .await
        .into_iter()
        .max_by_key(|status: &ProcStatus| status.code())
        .expect("no tasks to run")
}

#[cfg(test)]
mod tests {
    use ::std::time::Duration;

    use ::async_std::task::sleep;
    use ::futures::future::Either;
    use ::futures::future::select;
    use ::smallvec::smallvec;

    use super::*;

    #[async_std::test]
    async fn dependency_tree() {
        let top = Dependent::new(Task::noop(), smallvec![]);
        let mut mid1 = Dependent::new(Task::noop(), smallvec![]);
        mid1.depends_on(&top);
        let mut mid2 = Dependent::new(Task::noop(), smallvec![]);
        mid2.depends_on(&top);
        let mut botm = Dependent::new(Task::noop(), smallvec![]);
        botm.depends_on(&mid1);
        botm.depends_on(&mid2);
        let deps = vec![botm, mid1, top, mid2];
        match select(
                Box::pin(sleep(Duration::from_secs(3))),
                Box::pin(run_all(deps))
        ).await {
            Either::Left(status) => panic!("timeout"),
            Either::Right(status) => {}
        }
    }
}
