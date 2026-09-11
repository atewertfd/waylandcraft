package dev.evvie.waylandcraft.bridge;

import java.io.File;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

import dev.evvie.waylandcraft.desktop.RawDesktopEntry;

public class WaylandCraftBridge {
	private static native long init(long glfwGetProcAddress, long eglDisplay);
	private static native void shutdown(long instance);
	private static native void dispatchClients(long instance);
	private static native String socket(long instance);
	private static native long[] toplevels(long instance);
	private static native String toplevelTitle(long toplevelHandle);
	private static native String toplevelAppID(long toplevelHandle);
	private static native void updateSurfaceData(long instance, WLCSurface surface);
	private static native boolean execApp(long instance, String appId);
	private static native void freeToplevel(long instance, long toplevelHandle);
	private static native void keyboardFocus(long instance, long surfaceHandle);
	private static native void keyboardActivate(long instance);
	private static native void keyboardDeactivate(long instance);
	private static native void keyboardInput(long instance, int scancode, int action);
	private static native void pointerMotionFocus(long instance, long surfaceHandle, double x, double y);
	private static native int pointerButton(long instance, int button, int state);
	private static native void toplevelResize(long toplevelHandle, int width, int height, boolean interactive);
	private static native RawDesktopEntry[] loadDesktopEntries(long instance);

	private WLCSurface getOrCreateSurface(long handle) {
		return new WLCSurface(handle);
	}

	public static void main(String[] args) throws Exception {
		String dll = args.length > 0 ? args[0] : "waylandcraft.dll";
		boolean dumpOnly = args.length > 1 && "--dump".equals(args[1]);
		System.load(new File(dll).getAbsolutePath());
		System.out.println("PASS load " + dll);

		long instance = init(0, 0);
		if(instance == 0) {
			throw new IllegalStateException("FAIL init returned 0");
		}
		System.out.println("PASS init socket=" + socket(instance));

		RawDesktopEntry[] entries = loadDesktopEntries(instance);
		int running = 0;
		for(RawDesktopEntry entry : entries) {
			if(entry.appId != null && entry.appId.startsWith("hwnd:")) running++;
		}
		if(entries.length < 1) {
			throw new IllegalStateException("FAIL launcher returned no entries");
		}
		System.out.println("PASS launcher entries=" + entries.length + " running=" + running);
		if(dumpOnly) {
			for(RawDesktopEntry entry : entries) {
				if(entry.appId != null && entry.appId.startsWith("hwnd:")) {
					System.out.println("HWND " + entry.appId + " name=" + entry.name + " exec=" + entry.exec);
				}
			}
			shutdown(instance);
			return;
		}

		String first = firstMatching(entries, "notepad");
		String second = firstMatching(entries, "calculator");
		if(second == null) {
			second = firstMatching(entries, "opera");
		}
		if(second == null) {
			second = firstMatching(entries, "chrome");
		}
		if(second == null) {
			second = firstMatching(entries, "firefox");
		}
		if(second == null) {
			second = firstMatching(entries, "explorer");
		}
		if(second == null) {
			second = secondHwnd(entries, first);
		}
		if(args.length > 2 && "--capture".equals(args[1])) {
			first = "hwnd:" + args[2];
			if(args.length > 3) {
				second = "hwnd:" + args[3];
			}
		}

		Process launchedNotepad = null;
		if(first == null) {
			launchedNotepad = start("notepad.exe");
			first = waitForHwnd(instance, "notepad", 20000);
		}
		if(first == null) {
			throw new IllegalStateException("FAIL no Notepad HWND to adopt");
		}

		try {
			if(!execApp(instance, first)) {
				throw new IllegalStateException("FAIL adopt " + first);
			}
			System.out.println("PASS adopt " + first);

			long hwnd = parseHwnd(first);
			waitUntil(() -> {
				dispatchClients(instance);
				return contains(toplevels(instance), hwnd);
			}, 8000, "first adopted toplevel");
			WLCSurface surface = waitForFrame(instance, hwnd, 12000);
			assertLiveFrame(surface, "first");
			System.out.println("PASS first frame " + surface.width + "x" + surface.height + " attaches=" + surface.attachCount + " title=" + toplevelTitle(hwnd) + " app=" + toplevelAppID(hwnd));

			boolean noInput = false;
			for(String arg : args) {
				if("--no-input".equals(arg)) noInput = true;
			}
			boolean input = !noInput;
			if(input) {
				keyboardFocus(instance, hwnd);
				keyboardActivate(instance);
				pointerMotionFocus(instance, hwnd, 40, 40);
				pointerButton(instance, 0x110, 1);
				pointerButton(instance, 0x110, 0);
				typeScancodes(instance, new int[] { 0x1E, 0x30, 0x2E });
				keyboardDeactivate(instance);
				System.out.println("PASS first input posted");

				int beforeW = surface.width;
				int beforeH = surface.height;
				toplevelResize(hwnd, Math.max(640, beforeW - 20), Math.max(480, beforeH - 20), false);
				dispatchClients(instance);
				System.out.println("PASS first resize requested from " + beforeW + "x" + beforeH);
			} else {
				System.out.println("SKIP input/resize");
			}

			boolean oneWindow = args.length >= 3 && "--capture".equals(args[1]) && (args.length == 3 || noInput && args.length <= 4);
			if(oneWindow) {
				freeToplevel(instance, hwnd);
				System.out.println("PASS single-window capture");
				return;
			}
			if(second == null) {
				throw new IllegalStateException("FAIL no second window available (calculator/browser/explorer)");
			}
			if(!execApp(instance, second)) {
				throw new IllegalStateException("FAIL adopt second " + second);
			}
			long secondHandle = parseHwnd(second);
			waitUntil(() -> {
				dispatchClients(instance);
				return contains(toplevels(instance), secondHandle);
			}, 8000, "second adopted toplevel");
			WLCSurface secondSurface = waitForFrame(instance, secondHandle, 12000);
			assertLiveFrame(secondSurface, "second");
			if(toplevels(instance).length < 2) {
				throw new IllegalStateException("FAIL expected two adopted toplevels");
			}
			System.out.println("PASS two simultaneous captured windows first=" + toplevelTitle(hwnd) + " second=" + toplevelTitle(secondHandle));

			freeToplevel(instance, hwnd);
			waitUntil(() -> {
				dispatchClients(instance);
				return !contains(toplevels(instance), hwnd);
			}, 5000, "first unadopted");
			System.out.println("PASS first unadopted remaining=" + toplevels(instance).length);
			freeToplevel(instance, secondHandle);
		} finally {
			if(launchedNotepad != null) {
				launchedNotepad.destroyForcibly();
			}
			shutdown(instance);
			System.out.println("PASS shutdown");
		}
	}

	private static String firstMatching(RawDesktopEntry[] entries, String needle) {
		for(RawDesktopEntry entry : entries) {
			if(entry.appId != null && entry.appId.startsWith("hwnd:") && matches(entry, needle)) {
				return entry.appId;
			}
		}
		return null;
	}

	private static String secondHwnd(RawDesktopEntry[] entries, String first) {
		for(RawDesktopEntry entry : entries) {
			if(entry.appId != null && entry.appId.startsWith("hwnd:") && (first == null || !entry.appId.equals(first))) {
				return entry.appId;
			}
		}
		return null;
	}

	private static Process start(String command) throws Exception {
		Process process = new ProcessBuilder(command).redirectErrorStream(true).start();
		System.out.println("started " + command + " pid=" + process.pid());
		return process;
	}

	private static String waitForHwnd(long instance, String needle, long timeoutMs) throws Exception {
		return waitForHwnd(instance, needle, timeoutMs, 0);
	}

	private static String findHwnd(long instance, String needle, long timeoutMs) {
		try {
			return waitForHwnd(instance, needle, timeoutMs);
		} catch(Exception e) {
			System.out.println("WARN " + e.getMessage());
			return null;
		}
	}

	private static String waitForHwnd(long instance, String needle, long timeoutMs, long excludeHwnd) throws Exception {
		final String[] found = new String[1];
		waitUntil(() -> {
			dispatchClients(instance);
			for(RawDesktopEntry entry : loadDesktopEntries(instance)) {
				if(entry.appId == null || !entry.appId.startsWith("hwnd:") || !matches(entry, needle)) {
					continue;
				}
				if(excludeHwnd != 0 && parseHwnd(entry.appId) == excludeHwnd) {
					continue;
				}
				found[0] = entry.appId;
				return true;
			}
			return false;
		}, timeoutMs, "window matching " + needle);
		return found[0];
	}

	private static boolean matches(RawDesktopEntry entry, String needle) {
		String blob = ((entry.name == null ? "" : entry.name) + " " + (entry.exec == null ? "" : entry.exec) + " " + (entry.appId == null ? "" : entry.appId)).toLowerCase(Locale.ROOT);
		return blob.contains(needle.toLowerCase(Locale.ROOT));
	}

	private static long waitForToplevel(long instance, long timeoutMs) throws Exception {
		final long[] handle = new long[1];
		waitUntil(() -> {
			dispatchClients(instance);
			long[] tops = toplevels(instance);
			if(tops.length > 0) {
				handle[0] = tops[0];
				return true;
			}
			return false;
		}, timeoutMs, "adopted toplevel");
		return handle[0];
	}

	private static WLCSurface waitForFrame(long instance, long hwnd, long timeoutMs) throws Exception {
		WLCSurface surface = new WLCSurface(hwnd);
		waitUntil(() -> {
			dispatchClients(instance);
			updateSurfaceData(instance, surface);
			return surface.attachCount > 0 && surface.width > 1 && surface.height > 1 && surface.ptr != 0;
		}, timeoutMs, "WGC frame for HWND " + Long.toHexString(hwnd));
		return surface;
	}

	private static void assertLiveFrame(WLCSurface surface, String label) {
		if(surface.ptr == 0 || surface.width < 16 || surface.height < 16 || surface.stride < surface.width) {
			throw new IllegalStateException("FAIL " + label + " frame geometry ptr=" + surface.ptr + " " + surface.width + "x" + surface.height + " stride=" + surface.stride);
		}
		try {
			writeBmp(surface, label + ".bmp");
		} catch(Exception e) {
			System.out.println("WARN could not write " + label + ".bmp: " + e);
		}
		MemorySegment pixels = MemorySegment.ofAddress(surface.ptr).reinterpret((long) surface.stride * surface.height);
		int samples = 0;
		int nonzero = 0;
		int[] xs = { 2, surface.width / 2, surface.width - 3 };
		int[] ys = { 2, surface.height / 2, surface.height - 3 };
		List<String> sampleHex = new ArrayList<>();
		for(int y : ys) {
			for(int x : xs) {
				long offset = (long) y * surface.stride + (long) x * 4;
				int b = pixels.get(ValueLayout.JAVA_BYTE, offset) & 0xff;
				int g = pixels.get(ValueLayout.JAVA_BYTE, offset + 1) & 0xff;
				int r = pixels.get(ValueLayout.JAVA_BYTE, offset + 2) & 0xff;
				int a = pixels.get(ValueLayout.JAVA_BYTE, offset + 3) & 0xff;
				samples++;
				if((r | g | b | a) != 0) nonzero++;
				sampleHex.add(String.format("%02x%02x%02x%02x", r, g, b, a));
			}
		}
		if(nonzero == 0) {
			throw new IllegalStateException("FAIL " + label + " captured only zero pixels");
		}
		System.out.println("PASS " + label + " live pixels nonzero=" + nonzero + "/" + samples + " samples=" + sampleHex);
	}

	private static void typeScancodes(long instance, int[] scancodes) throws InterruptedException {
		for(int scancode : scancodes) {
			keyboardInput(instance, scancode, 1);
			Thread.sleep(30);
			keyboardInput(instance, scancode, 0);
			Thread.sleep(30);
		}
	}

	private static void waitUntil(Check check, long timeoutMs, String what) throws Exception {
		long deadline = System.currentTimeMillis() + timeoutMs;
		Exception last = null;
		while(System.currentTimeMillis() < deadline) {
			try {
				if(check.ok()) {
					return;
				}
			} catch(Exception e) {
				last = e;
			}
			Thread.sleep(50);
		}
		if(last != null) {
			throw new IllegalStateException("FAIL timeout waiting for " + what, last);
		}
		throw new IllegalStateException("FAIL timeout waiting for " + what);
	}

	private static boolean contains(long[] values, long needle) {
		for(long value : values) {
			if(value == needle) return true;
		}
		return false;
	}

	private static void writeBmp(WLCSurface surface, String name) throws Exception {
		int width = surface.width;
		int height = surface.height;
		int row = width * 3;
		int pad = (4 - (row % 4)) % 4;
		int img = (row + pad) * height;
		java.nio.ByteBuffer header = java.nio.ByteBuffer.allocate(54).order(java.nio.ByteOrder.LITTLE_ENDIAN);
		header.put((byte) 'B').put((byte) 'M');
		header.putInt(54 + img);
		header.putInt(0);
		header.putInt(54);
		header.putInt(40);
		header.putInt(width);
		header.putInt(height);
		header.putShort((short) 1);
		header.putShort((short) 24);
		header.putInt(0);
		header.putInt(img);
		header.putInt(2835);
		header.putInt(2835);
		header.putInt(0);
		header.putInt(0);
		MemorySegment pixels = MemorySegment.ofAddress(surface.ptr).reinterpret((long) surface.stride * height);
		try(java.io.FileOutputStream out = new java.io.FileOutputStream(name)) {
			out.write(header.array());
			byte[] line = new byte[row + pad];
			for(int y = height - 1; y >= 0; y--) {
				for(int x = 0; x < width; x++) {
					long offset = (long) y * surface.stride + (long) x * 4;
					line[x * 3] = pixels.get(ValueLayout.JAVA_BYTE, offset);
					line[x * 3 + 1] = pixels.get(ValueLayout.JAVA_BYTE, offset + 1);
					line[x * 3 + 2] = pixels.get(ValueLayout.JAVA_BYTE, offset + 2);
				}
				out.write(line);
			}
		}
		System.out.println("wrote " + new java.io.File(name).getAbsolutePath() + " " + width + "x" + height);
	}

	private static long parseHwnd(String appId) {
		return Long.parseLong(appId.substring("hwnd:".length()));
	}

	@FunctionalInterface
	private interface Check {
		boolean ok() throws Exception;
	}
}
