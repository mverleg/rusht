use ::std::collections::HashMap;
use ::std::process::Command;
use ::std::process::Stdio;

#[async_std::main]
async fn main() {
    // JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-17.arm64.jdk/Contents/Home MAVEN_OPTS='-XX:+UseG1GC -Xms256m -Xmx8192m' time /opt/homebrew/bin/mvn test-compile --threads=10 --offline --quiet -Djava.net.preferIPv4Stack=true -Dmanagedversions.skip=true -Dmanagedversions.failOnError=false -Denforcer.skip=true -Ddatabase.skip=true -Dmaven.javadoc.skip=true -DskipTests=true --activate-profiles='!modules/all,!system,modules/viper'
    let mut env = HashMap::new();
    env.insert("JAVA_HOME", "/Library/Java/JavaVirtualMachines/temurin-17.arm64.jdk/Contents/Home");
    env.insert("MAVEN_OPTS", "-XX:+UseG1GC -Xms256m -Xmx8192m");
    // let mut child1 = Command::new("/opt/homebrew/bin/mvn")
    //     .args(&vec!["test-compile", "--threads=10", "--offline", "--quiet", "-Djava.net.preferIPv4Stack=true", "-Dmanagedversions.skip=true", "-Dmanagedversions.failOnError=false", "-Denforcer.skip=true", "-Ddatabase.skip=true", "-Dmaven.javadoc.skip=true", "-DskipTests=true", "--activate-profiles='!modules/all,!system,modules/viper'"])
    //     .envs(&env)
    //     .stdout(Stdio::inherit())
    //     .stderr(Stdio::inherit())
    //     .spawn()
    //     .unwrap();
    // child1.wait().unwrap();
    //TODO @mverleg: slow^
    let mut child2 = Command::new("sh")
        .args(&vec!["-c", "JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-17.arm64.jdk/Contents/Home MAVEN_OPTS='-XX:+UseG1GC -Xms256m -Xmx8192m' /opt/homebrew/bin/mvn test-compile --threads=10 --offline --quiet -Djava.net.preferIPv4Stack=true -Dmanagedversions.skip=true -Dmanagedversions.failOnError=false -Denforcer.skip=true -Ddatabase.skip=true -Dmaven.javadoc.skip=true -DskipTests=true --activate-profiles='!modules/all,!system,modules/viper'"])
        .spawn()
        .unwrap();
    //TODO @mverleg: fast^
    child2.wait().unwrap();
}
