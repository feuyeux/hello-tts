# Java TTS Implementation

This directory contains the Java implementation of the hello-edge-tts project using Java 21 LTS with modern HTTP client and CompletableFuture for async operations. This implementation showcases the latest Java features, reactive programming patterns, virtual threads, and enterprise-grade error handling.

## 🚀 Quick Start

```bash
# Navigate to Java directory
cd hello-tts-java
mvn compile

mvn compile; chcp 65001; mvn exec:java "-Dexec.mainClass=org.feuyeux.tts.HelloTTS" "-Dexec.jvmArgs=-Dfile.encoding=UTF-8"

mvn exec:java -Dexec.mainClass="org.feuyeux.tts.HelloTTS"
mvn exec:java -Dexec.mainClass="org.feuyeux.tts.HelloTTS" -Dexec.args="--backend edge --text 你好世界 --voice zh-CN-XiaoxiaoNeural"
mvn exec:java -Dexec.mainClass="org.feuyeux.tts.HelloTTS" -Dexec.args="--backend google --text 'Hello World' --voice en"
mvn exec:java -Dexec.mainClass="org.feuyeux.tts.HelloTTS" -Dexec.args="--backend google --text '你好世界' --voice zh"
mvn exec:java -Dexec.mainClass="org.feuyeux.tts.HelloTTS" -Dexec.args="--list-voices"

mvn clean package
java -jar target/hello-edge-tts-standalone.jar --help
```

```bash
mvn exec:java -Dexec.mainClass="org.feuyeux.tts.HelloMultilingual" -Dexec.args="--backend google"
```