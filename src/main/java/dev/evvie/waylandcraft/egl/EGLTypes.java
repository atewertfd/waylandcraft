package dev.evvie.waylandcraft.egl;

public class EGLTypes {
	
	public static class EGLDisplay {
		private final long handle;
		private EGLDisplay(long handle) {
			this.handle = handle;
		}
		public static EGLDisplay of(long handle) {
			return handle == 0 ? null : new EGLDisplay(handle);
		}
		public static long address(EGLDisplay obj) {
			return obj == null ? 0 : obj.handle;
		}
	}
	
	public static class EGLContext {
		private final long handle;
		private EGLContext(long handle) {
			this.handle = handle;
		}
		public static EGLContext of(long handle) {
			return handle == 0 ? null : new EGLContext(handle);
		}
		public static long address(EGLContext obj) {
			return obj == null ? 0 : obj.handle;
		}
	}
	
	public static class EGLDeviceEXT {
		private final long handle;
		private EGLDeviceEXT(long handle) {
			this.handle = handle;
		}
		public static EGLDeviceEXT of(long handle) {
			return handle == 0 ? null : new EGLDeviceEXT(handle);
		}
		public static long address(EGLDeviceEXT obj) {
			return obj == null ? 0 : obj.handle;
		}
	}
	
	public static class EGLImage {
		private final long handle;
		private EGLImage(long handle) {
			this.handle = handle;
		}
		public static EGLImage of(long handle) {
			return handle == 0 ? null : new EGLImage(handle);
		}
		public static long address(EGLImage obj) {
			return obj == null ? 0 : obj.handle;
		}
	}
	
	public static class EGLClientBuffer {
		private final long handle;
		private EGLClientBuffer(long handle) {
			this.handle = handle;
		}
		public static EGLClientBuffer of(long handle) {
			return handle == 0 ? null : new EGLClientBuffer(handle);
		}
		public static long address(EGLClientBuffer obj) {
			return obj == null ? 0 : obj.handle;
		}
	}
	
}
