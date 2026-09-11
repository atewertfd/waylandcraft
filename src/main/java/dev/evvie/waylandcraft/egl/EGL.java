package dev.evvie.waylandcraft.egl;

import java.nio.IntBuffer;
import java.nio.LongBuffer;

import org.lwjgl.PointerBuffer;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.system.JNI;
import org.lwjgl.system.MemoryUtil;

import dev.evvie.waylandcraft.egl.EGLTypes.EGLClientBuffer;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLContext;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLDeviceEXT;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLDisplay;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLImage;

public class EGL {
	
	public static final int EGL_TRUE = 1;
	public static final int EGL_FALSE = 0;
	public static final long EGL_NONE = 0x3038;
	public static final EGLContext EGL_NO_CONTEXT = null;
	public static final EGLImage EGL_NO_IMAGE = null;
	public static final long EGL_WIDTH = 0x3057;
	public static final long EGL_HEIGHT = 0x3056;
	public static final long EGL_LINUX_DMA_BUF_EXT = 0x3270;
	public static final long EGL_LINUX_DRM_FOURCC_EXT = 0x3271;
	public static final int EGL_DEVICE_EXT = 0x322C;
	
	public static final long EGL_DMA_BUF_PLANE0_FD_EXT = 0x3272;
	public static final long EGL_DMA_BUF_PLANE0_OFFSET_EXT = 0x3273;
	public static final long EGL_DMA_BUF_PLANE0_PITCH_EXT = 0x3274;
	public static final long EGL_DMA_BUF_PLANE1_FD_EXT = 0x3275;
	public static final long EGL_DMA_BUF_PLANE1_OFFSET_EXT = 0x3276;
	public static final long EGL_DMA_BUF_PLANE1_PITCH_EXT = 0x3277;
	public static final long EGL_DMA_BUF_PLANE2_FD_EXT = 0x3278;
	public static final long EGL_DMA_BUF_PLANE2_OFFSET_EXT = 0x3279;
	public static final long EGL_DMA_BUF_PLANE2_PITCH_EXT = 0x327A;
	public static final long EGL_DMA_BUF_PLANE3_FD_EXT = 0x3440;
	public static final long EGL_DMA_BUF_PLANE3_OFFSET_EXT = 0x3441;
	public static final long EGL_DMA_BUF_PLANE3_PITCH_EXT = 0x3442;
	
	public static final long EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT = 0x3443;
	public static final long EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT = 0x3444;
	public static final long EGL_DMA_BUF_PLANE1_MODIFIER_LO_EXT = 0x3445;
	public static final long EGL_DMA_BUF_PLANE1_MODIFIER_HI_EXT = 0x3446;
	public static final long EGL_DMA_BUF_PLANE2_MODIFIER_LO_EXT = 0x3447;
	public static final long EGL_DMA_BUF_PLANE2_MODIFIER_HI_EXT = 0x3448;
	public static final long EGL_DMA_BUF_PLANE3_MODIFIER_LO_EXT = 0x3449;
	public static final long EGL_DMA_BUF_PLANE3_MODIFIER_HI_EXT = 0x344A;
	
	public static final int EGL_DRM_RENDER_NODE_FILE_EXT = 0x3377;
	public static final int EGL_DRM_DEVICE_FILE_EXT = 0x3233;
	
	private static long getProcAddress(String name) {
		return GLFW.glfwGetProcAddress(name);
	}
	
	private static long procQueryDisplayAttribEXT;
	private static long procQueryDeviceStringEXT;
	private static long procCreateImage;
	private static long procGetError;
	private static long procQueryDmaBufFormatsEXT;
	private static long procQueryDmaBufModifiersEXT;
	
	static {
		procQueryDisplayAttribEXT = getProcAddress("eglQueryDisplayAttribEXT");
		procQueryDeviceStringEXT = getProcAddress("eglQueryDeviceStringEXT");
		procCreateImage = getProcAddress("eglCreateImage");
		procGetError = getProcAddress("eglGetError");
		procQueryDmaBufFormatsEXT = getProcAddress("eglQueryDmaBufFormatsEXT");
		procQueryDmaBufModifiersEXT = getProcAddress("eglQueryDmaBufModifiersEXT");
	}
	
	/* EGL types
	 * 
	 * EGLBoolean -> int
	 * EGLenum -> int
	 * EGLDisplay -> void*
	 * EGLContext -> void*
	 * EGLDeviceEXT -> void*
	 * EGLImage -> void*
	 * EGLClientBuffer -> void*
	 * EGLint -> int
	 * EGLAttrib -> void*
	 * EGLuint64KHR -> long
	 */
	
	public static boolean eglQueryDisplayAttribEXT(EGLDisplay dpy, int name, PointerBuffer ptr) {
		// EGLBoolean eglQueryDisplayAttribEXT (EGLDisplay dpy, EGLint name, EGLAttrib *value);
		return JNI.invokePPI(EGLDisplay.address(dpy), name, MemoryUtil.memAddressSafe(ptr), procQueryDisplayAttribEXT) == EGL_TRUE;
	}
	
	public static String eglQueryDeviceStringEXT(EGLDeviceEXT device, int name) {
		// const char * eglQueryDeviceStringEXT (EGLDeviceEXT device, EGLint name);
		long str = JNI.invokePP(EGLDeviceEXT.address(device), name, procQueryDeviceStringEXT);
		return MemoryUtil.memUTF8Safe(str);
	}
	
	public static EGLImage eglCreateImage(EGLDisplay display, EGLContext context, int target, EGLClientBuffer buffer, PointerBuffer attrib_list) {
		// EGLImage eglCreateImage (EGLDisplay dpy, EGLContext ctx, EGLenum target, EGLClientBuffer buffer, const EGLAttrib *attrib_list);
		long handle = JNI.invokePPPPP(EGLDisplay.address(display), EGLContext.address(context), target, EGLClientBuffer.address(buffer), MemoryUtil.memAddressSafe(attrib_list), procCreateImage);
		return EGLImage.of(handle);
	}
	
	public static int eglGetError() {
		// EGLint eglGetError (void);
		return JNI.invokeI(procGetError);
	}
	
	public static boolean eglQueryDmaBufFormatsEXT(EGLDisplay dpy, int max_formats, IntBuffer formats, IntBuffer num_formats) {
		// EGLAPI EGLBoolean EGLAPIENTRY eglQueryDmaBufFormatsEXT (EGLDisplay dpy, EGLint max_formats, EGLint *formats, EGLint *num_formats);
		return JNI.invokePPPI(EGLDisplay.address(dpy), max_formats, MemoryUtil.memAddressSafe(formats), MemoryUtil.memAddressSafe(num_formats), procQueryDmaBufFormatsEXT) == EGL_TRUE;
	}
	
	public static boolean eglQueryDmaBufModifiersEXT(EGLDisplay dpy, int format, int max_modifiers, LongBuffer modifiers, IntBuffer external_only, IntBuffer num_modifiers) {
		// EGLBoolean eglQueryDmaBufModifiersEXT (EGLDisplay dpy, EGLint format, EGLint max_modifiers, EGLuint64KHR *modifiers, EGLBoolean *external_only, EGLint *num_modifiers);
		return JNI.invokePPPPI(EGLDisplay.address(dpy), format, max_modifiers, MemoryUtil.memAddressSafe(modifiers), MemoryUtil.memAddressSafe(external_only), MemoryUtil.memAddressSafe(num_modifiers), procQueryDmaBufModifiersEXT) == EGL_TRUE;
	}
	
}
