use ::std::collections::HashMap;
use ::std::process::Command;
use ::std::process::Stdio;

#[async_std::main]
async fn main() {
    let mut env = HashMap::new();
    env.insert("JAVA_HOME", "/Library/Java/JavaVirtualMachines/temurin-17.arm64.jdk/Contents/Home");
    env.insert("MAVEN_OPTS", "-XX:+UseG1GC -Xms256m -Xmx8192m");
    let mut child = Command::new("/opt/homebrew/bin/mvn")
        .args(&vec!["test-compile", "--threads=10", "--offline", "--quiet", "-Djava.net.preferIPv4Stack=true", "-Dmanagedversions.skip=true", "-Dmanagedversions.failOnError=false", "-Denforcer.skip=true", "-Ddatabase.skip=true", "-Dmaven.javadoc.skip=true", "-DskipTests=true", "--activate-profiles='!modules/all,!system,modules/viper'"])
        .envs(&env)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    child.wait().unwrap();
}
