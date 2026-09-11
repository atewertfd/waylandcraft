# Windows JNI smoke

Standalone Java harness that loads the Windows native library without Minecraft.

```bat
set JAVA_HOME=C:\Program Files\Java\jdk-25.0.4
javac --release 25 dev\evvie\waylandcraft\desktop\RawDesktopEntry.java dev\evvie\waylandcraft\bridge\WLCSurface.java dev\evvie\waylandcraft\bridge\WaylandCraftBridge.java
java --enable-native-access=ALL-UNNAMED -cp . dev.evvie.waylandcraft.bridge.WaylandCraftBridge ..\..\native\target\debug\waylandcraft.dll
```

`--dump` lists adopted-candidate HWNDs. `--capture <hwnd> <hwnd>` adopts specific windows.
