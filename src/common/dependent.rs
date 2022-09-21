use ::std::rc::Rc;
use ::std::sync::atomic::AtomicU64;
use ::std::sync::atomic::Ordering;
use ::std::time::Duration;

use ::futures::future::join_all;
use ::log::debug;
use ::smallvec::smallvec;
use ::smallvec::SmallVec;

use crate::common::async_gate::AsyncGate;
use crate::common::write::FunnelFactory;
use crate::common::{LineWriter, Task};
use crate::ExitStatus;

static DEBUG_NR: AtomicU64 = AtomicU64::new(0); //TODO @mverleg:

#[derive(Debug)]
pub struct Dependency {
    name: Rc<String>,
    gate: AsyncGate,
}

impl Dependency {
    #[allow(dead_code)]
    pub fn new_with_gate(name: Rc<String>, gate: AsyncGate) -> Self {
        Dependency { name, gate }
    }
}

#[derive(Debug)]
pub struct Dependent {
    task: Option<Task>,
    name: Rc<String>,
    current: AsyncGate,
    dependencies: SmallVec<[Dependency; 1]>,
}

impl Dependent {
    pub fn new_optional(name: impl Into<String>, task: Option<Task>) -> Self {
        Dependent {
            task,
            name: Rc::new(name.into()),
            current: AsyncGate::new(),
            dependencies: smallvec![],
        }
    }

    pub fn new_named(name: impl Into<String>, task: Task) -> Self {
        Dependent::new_optional(name, Some(task))
    }

    pub fn new_noop(name: impl Into<String>) -> Self {
        Dependent::new_optional(name, None)
    }

    pub fn new(task: Task) -> Self {
        let name = task.as_str();
        Dependent::new_named(name, task)
    }

    pub fn depends_on(&mut self, other: &Dependent) {
        //TODO @mverleg: might skip non-task deps, but only if the tree is always build ascending...
        self.dependencies.push(Dependency {
            name: other.name.clone(),
            gate: other.current.clone(),
        })
    }

    // Note this takes owned LineWriter instead of &mut because of run_all. Try using e.g. `FunnelWriter`.
    pub async fn await_and_exec(&self, mut writer: impl LineWriter) -> ExitStatus {
        let count = self.dependencies.len();
        for (nr, dependency) in self.dependencies.iter().enumerate() {
            if dependency.gate.is_open() {
                debug!(
                    "{} needs {} [{}/{}] which is immediately available",
                    self.name,
                    dependency.name,
                    nr + 1,
                    count
                );
            } else {
                debug!(
                    "{} needs {} [{}/{}] which needs to be awaited",
                    self.name,
                    dependency.name,
                    nr + 1,
                    count
                );
                let nr = DEBUG_NR.fetch_add(1, Ordering::AcqRel); //TODO @mverleg: TEMPORARY! REMOVE THIS!
                debug!("{} before {} waits for {}", nr, self.name, dependency.name); //TODO @mverleg: TEMPORARY! REMOVE THIS!
                let timeout = Duration::from_secs(150);
                match dependency.gate.wait_timeout(&timeout).await {
                    //TODO @mverleg: make timeout configurable
                    Ok(true) => {
                        debug!(
                            "{} was waiting for {} [{}/{}] which just completed",
                            self.name,
                            dependency.name,
                            nr + 1,
                            count
                        );
                    }
                    Ok(false) => {
                        debug!(
                            "{} was waiting for {} [{}/{}] which just failed! skipping execution",
                            self.name,
                            dependency.name,
                            nr + 1,
                            count
                        );
                        self.current.open(false);
                        return ExitStatus::err();
                    }
                    Err(()) => {
                        debug!(
                            "{} was waiting for {} [{}/{}] but it timed out ({} s)",
                            self.name,
                            dependency.name,
                            nr + 1,
                            count,
                            &timeout.as_secs(),
                        );
                        self.current.open(false);
                        return ExitStatus::err();
                    }
                }
                debug!("{} after {} waits for {}", nr, self.name, dependency.name);
                //TODO @mverleg: TEMPORARY! REMOVE THIS!
            }
        }
        if let Some(task) = &self.task {
            self.current.open(false);
            task.execute_with_stdout(true, &mut writer).await
        } else {
            self.current.open(true);
            ExitStatus::ok()
        }
    }

    pub fn task(&self) -> Option<&Task> {
        self.task.as_ref()
    }
}

pub async fn run_all(dependents: Vec<Dependent>, writer: &mut impl LineWriter) -> ExitStatus {
    let fac = FunnelFactory::new(writer);
    join_all(
        dependents
            .iter()
            .map(|dep| dep.await_and_exec(fac.writer(dep.name.as_ref())))
            .collect::<Vec<_>>(),
    )
    .await
    .into_iter()
    .max_by_key(|status| status.code)
    .expect("no tasks to run")
}

#[cfg(test)]
mod tests {
    use ::std::time::Duration;

    use ::async_std::task::sleep;
    use ::futures::future::select;
    use ::futures::future::Either;

    use crate::common::StdWriter;

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
            Box::pin(run_all(deps, &mut StdWriter::stdout())),
        )
        .await
        {
            Either::Left(_) => panic!("timeout"),
            Either::Right(_) => {}
        }
    }
}
