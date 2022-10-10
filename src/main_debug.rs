use ::std::env;
use ::std::path::PathBuf;

use ::std::process::Command;
use ::std::thread::sleep;
use ::std::time::Duration;

use ::rusht::common::RejectStdin;

fn main() {
    // use cpal::traits::{DeviceTrait, HostTrait};
    // let mut supported_configs_range = device
    //     .supported_output_configs()
    //     .expect("error while querying configs");
    // let supported_config = supported_configs_range
    //     .next()
    //     .expect("no supported config?!")
    //     .with_max_sample_rate();
}

// The conclusion here is that running Command mvn is slow, compared to running `sh -c "mvn ..."`
// * Copying all the end does not help.
// * There appears more logging, perhaps mvn is doing unnecessary work.
// * It seems faster than clean build, so some cache is picked up.

// #[async_std::main]
// async fn main_old() {
//     let _reader = RejectStdin::new();
//     sleep(Duration::from_secs(2));
//     println!("done");
//
//     // JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-17.arm64.jdk/Contents/Home MAVEN_OPTS='-XX:+UseG1GC -Xms256m -Xmx8192m' time /opt/homebrew/bin/mvn test-compile --threads=10 --offline --quiet -Djava.net.preferIPv4Stack=true -Dmanagedversions.skip=true -Dmanagedversions.failOnError=false -Denforcer.skip=true -Ddatabase.skip=true -Dmaven.javadoc.skip=true -DskipTests=true --activate-profiles='!modules/all,!system,modules/viper'
//     // let mut env = HashMap::new();
//     // let path = env::var("PATH").unwrap();
//     // let ld_path = env::var("LD_LIBRARY_PATH").unwrap();
//     // env.insert("JAVA_HOME", "/Library/Java/JavaVirtualMachines/temurin-17.arm64.jdk/Contents/Home");
//     // env.insert("MAVEN_OPTS", "-XX:+UseG1GC -Xms256m -Xmx8192m");
//     // env.insert("PATH", &path);
//     // env.insert("LD_LIBRARY_PATH", &ld_path);
//     let env = env::vars();
//     let mut child1 = Command::new("/opt/homebrew/bin/mvn")
//         .args(&vec![
//             "test-compile",
//             "--threads=10",
//             "--offline",
//             "--quiet",
//             "-Djava.net.preferIPv4Stack=true",
//             "-Dmanagedversions.skip=true",
//             "-Dmanagedversions.failOnError=false",
//             "-Denforcer.skip=true",
//             "-Ddatabase.skip=true",
//             "-Dmaven.javadoc.skip=true",
//             "-DskipTests=true",
//             "--activate-profiles='!modules/all,!system,modules/viper'",
//         ])
//         .env_clear()
//         .envs(env)
//         .current_dir(PathBuf::from("/Users/mverleg/data/goat"))
//         // .stdout(Stdio::inherit())
//         // .stderr(Stdio::inherit())
//         .spawn()
//         .unwrap();
//     child1.wait().unwrap();
//     //TODO @mverleg: slow^
//     // let mut child2 = Command::new("sh")
//     //     .args(&vec!["-c", "JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-17.arm64.jdk/Contents/Home MAVEN_OPTS='-XX:+UseG1GC -Xms256m -Xmx8192m' /opt/homebrew/bin/mvn test-compile --threads=10 --offline --quiet -Djava.net.preferIPv4Stack=true -Dmanagedversions.skip=true -Dmanagedversions.failOnError=false -Denforcer.skip=true -Ddatabase.skip=true -Dmaven.javadoc.skip=true -DskipTests=true --activate-profiles='!modules/all,!system,modules/viper'"])
//     //     .spawn()
//     //     .unwrap();
//     // child2.wait().unwrap();
//     //TODO @mverleg: fast^
// }
