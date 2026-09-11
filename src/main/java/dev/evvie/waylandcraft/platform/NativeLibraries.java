package dev.evvie.waylandcraft.platform;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;

import org.lwjgl.system.Platform;

import dev.evvie.waylandcraft.WaylandCraftCommon;

/**
 * Extracts and loads the correct platform native library from the mod JAR.
 */
public final class NativeLibraries {

	private NativeLibraries() {
	}

	public static void load() {
		List<String> candidates = candidateResourcePaths();
		IOException lastIo = null;

		for(String path : candidates) {
			WaylandCraftCommon.LOGGER.info("Looking for native library '{}'", path);
			try(InputStream stream = NativeLibraries.class.getResourceAsStream(path)) {
				if(stream == null) {
					continue;
				}
				File temp = extract(stream, suffixFor(path));
				System.load(temp.getAbsolutePath());
				WaylandCraftCommon.LOGGER.info("Loaded native library from jar: {} -> {}", path, temp.getAbsolutePath());
				return;
			} catch(UnsatisfiedLinkError e) {
				throw new NativeLibraryLoadException("Found " + path + " but the OS rejected it. Backend=" + PlatformSupport.backendName(), e);
			} catch(IOException e) {
				lastIo = e;
				WaylandCraftCommon.LOGGER.warn("Failed to extract {}", path, e);
			}
		}

		WaylandCraftCommon.LOGGER.info("Native library could not be loaded from jar. Attempting System.loadLibrary(\"waylandcraft\")");
		try {
			System.loadLibrary("waylandcraft");
			WaylandCraftCommon.LOGGER.info("Loaded native library from java.library.path");
		} catch(UnsatisfiedLinkError e) {
			String message = missingLibraryMessage(candidates, lastIo);
			throw new NativeLibraryLoadException(message, e);
		}
	}

	public static List<String> candidateResourcePaths() {
		ArrayList<String> paths = new ArrayList<>();
		Platform platform = Platform.get();
		Platform.Architecture arch = Platform.getArchitecture();

		if(platform == Platform.WINDOWS) {
			if(arch == Platform.Architecture.X64) {
				paths.add("/waylandcraft-windows-msvc-x86_64.dll");
			}
			else if(arch == Platform.Architecture.ARM64) {
				paths.add("/waylandcraft-windows-msvc-arm64.dll");
			}
			paths.add("/waylandcraft.dll");
		}
		else if(platform == Platform.LINUX) {
			paths.add("/libwaylandcraft.so");
			String linuxArch = switch(arch) {
				case X64 -> "x86_64";
				case ARM64 -> "arm64";
				default -> null;
			};
			if(linuxArch != null) {
				paths.add("/libwaylandcraft-linux-gnu-" + linuxArch + ".so");
			}
		}

		return paths;
	}

	private static String suffixFor(String resourcePath) {
		if(resourcePath.endsWith(".dll")) return "-waylandcraft.dll";
		if(resourcePath.endsWith(".so")) return "-libwaylandcraft.so";
		return "-waylandcraft.bin";
	}

	private static File extract(InputStream inputStream, String suffix) throws IOException {
		byte[] data = inputStream.readAllBytes();
		File temp = File.createTempFile("waylandcraft-", suffix);
		temp.deleteOnExit();
		try(FileOutputStream outputStream = new FileOutputStream(temp)) {
			outputStream.write(data);
		}
		try {
			Files.setPosixFilePermissions(temp.toPath(), java.nio.file.attribute.PosixFilePermissions.fromString("rwxr-xr-x"));
		} catch(UnsupportedOperationException ignored) {
			// Windows temp files do not use POSIX permissions.
		}
		return temp;
	}

	private static String missingLibraryMessage(List<String> candidates, IOException lastIo) {
		StringBuilder builder = new StringBuilder();
		builder.append("WaylandCraft could not load a native library for ")
			.append(Platform.get())
			.append(' ')
			.append(Platform.getArchitecture())
			.append(" (backend=")
			.append(PlatformSupport.backendName())
			.append(", version=")
			.append(PlatformSupport.nativeLibraryVersion())
			.append("). Looked for: ")
			.append(candidates);
		if(lastIo != null) {
			builder.append(". Last extract error: ").append(lastIo.getMessage());
		}
		builder.append(". Rebuild with cargo + gradlew.bat build so the DLL/SO is packaged inside the JAR.");
		return builder.toString();
	}

	public static final class NativeLibraryLoadException extends RuntimeException {
		public NativeLibraryLoadException(String message, Throwable cause) {
			super(message, cause);
		}
	}
}
