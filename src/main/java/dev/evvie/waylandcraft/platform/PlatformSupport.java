package dev.evvie.waylandcraft.platform;

import org.lwjgl.system.Platform;

import com.mojang.blaze3d.systems.RenderSystem;

import dev.evvie.waylandcraft.WaylandCraftCommon;

/**
 * Platform and renderer detection for the unofficial Windows port.
 * Linux keeps the existing Wayland compositor; Windows uses native capture.
 */
public final class PlatformSupport {

	public enum Backend {
		LINUX_WAYLAND,
		WINDOWS_CAPTURE,
		UNSUPPORTED
	}

	private PlatformSupport() {
	}

	public static boolean isLinux() {
		return Platform.get() == Platform.LINUX;
	}

	public static boolean isWindows() {
		return Platform.get() == Platform.WINDOWS;
	}

	public static boolean isSupportedOs() {
		return isLinux() || isWindows();
	}

	public static Backend detectBackend() {
		if(isLinux()) return Backend.LINUX_WAYLAND;
		if(isWindows()) return Backend.WINDOWS_CAPTURE;
		return Backend.UNSUPPORTED;
	}

	public static String backendName() {
		return switch(detectBackend()) {
			case LINUX_WAYLAND -> "linux-wayland";
			case WINDOWS_CAPTURE -> "windows-graphics-capture";
			case UNSUPPORTED -> "unsupported";
		};
	}

	public static String nativeLibraryVersion() {
		return "2.1.0-windows.26.2-alpha.1";
	}

	/**
	 * Minecraft 26.2 can select an experimental Vulkan backend. This port's
	 * compatibility texture upload still uses the default OpenGL backend.
	 */
	public static boolean isVulkanRenderer() {
		try {
			if(!RenderSystem.isOnRenderThread()) {
				return false;
			}
			var device = RenderSystem.getDevice();
			if(device == null) {
				return false;
			}
			String name = device.getClass().getName();
			String label = device.toString();
			boolean vulkan = containsVulkan(name) || containsVulkan(label);
			if(vulkan) {
				WaylandCraftCommon.LOGGER.error("Detected Vulkan GpuDevice ({}). WaylandCraft window textures require the default OpenGL renderer.", name);
			}
			else {
				WaylandCraftCommon.LOGGER.info("GpuDevice: {} renderer={}", name, label);
			}
			return vulkan;
		} catch(Throwable t) {
			WaylandCraftCommon.LOGGER.warn("Could not inspect GpuDevice: {}", t.toString());
			return false;
		}
	}

	private static boolean containsVulkan(String value) {
		if(value == null) return false;
		String lower = value.toLowerCase();
		return lower.contains("vulkan") || lower.contains("vkdevice") || lower.contains(".vk.");
	}
}
