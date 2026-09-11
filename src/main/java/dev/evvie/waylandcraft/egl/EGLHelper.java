package dev.evvie.waylandcraft.egl;

import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;

import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryUtil;

import dev.evvie.waylandcraft.egl.EGLTypes.EGLDeviceEXT;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLDisplay;

public class EGLHelper {
	
	public static String getRenderNodePath(EGLDisplay dpy) throws EGLError {
		PointerBuffer deviceRet = MemoryUtil.memAllocPointer(1);
		if(!EGL.eglQueryDisplayAttribEXT(dpy, EGL.EGL_DEVICE_EXT, deviceRet)) {
			throw new EGLError();
		}
		
		EGLDeviceEXT device = EGLDeviceEXT.of(deviceRet.get());
		
		String path = EGL.eglQueryDeviceStringEXT(device, EGL.EGL_DRM_RENDER_NODE_FILE_EXT);
		return path;
	}
	
	public static int[] queryDmabufFormatCodes(EGLDisplay dpy) throws EGLError {
		IntBuffer numRet = MemoryUtil.memAllocInt(1);
		if(!EGL.eglQueryDmaBufFormatsEXT(dpy, 0, null, numRet)) {
			throw new EGLError();
		}
		
		int num = numRet.get();
		IntBuffer formatsBuf = MemoryUtil.memAllocInt(num);
		if(!EGL.eglQueryDmaBufFormatsEXT(dpy, num, formatsBuf, numRet)) {
			throw new EGLError();
		}
		
		int[] formats = new int[num];
		formatsBuf.get(formats);
		return formats;
	}
	
	public static ArrayList<Long> queryDmabufModifiers(EGLDisplay dpy, int format) throws EGLError {
		IntBuffer countRet = MemoryUtil.memAllocInt(1);
		if(!EGL.eglQueryDmaBufModifiersEXT(dpy, format, 0, null, null, countRet)) {
			throw new EGLError();
		}
		
		int count = countRet.get();
		LongBuffer modifiersBuf = MemoryUtil.memAllocLong(count);
		IntBuffer externalBuf = MemoryUtil.memAllocInt(count);
		if(!EGL.eglQueryDmaBufModifiersEXT(dpy, format, count, modifiersBuf, externalBuf, countRet)) {
			throw new EGLError();
		}
		
		ArrayList<Long> modifiers = new ArrayList<Long>();
		for(int i = 0; i < count; i++) {
			long m = modifiersBuf.get();
			int e = externalBuf.get();
			
			if(e == EGL.EGL_TRUE) continue;
			modifiers.add(m);
		}
		
		return modifiers;
	}
	
	public static ArrayList<DmabufFormat> queryDmabufFormats(EGLDisplay dpy) throws EGLError {
		ArrayList<DmabufFormat> formats = new ArrayList<DmabufFormat>();
		int[] codes = queryDmabufFormatCodes(dpy);
		
		for(int code : codes) {
			formats.add(new DmabufFormat(code, DRM_MODIFIER_INVALID));
		}
		for(int code : codes) {
			ArrayList<Long> mods = queryDmabufModifiers(dpy, code);
			for(long mod : mods) {
				formats.add(new DmabufFormat(code, mod));
			}
		}
		
		return formats;
	}
	
	public static final long DRM_MODIFIER_INVALID = 0xffffffffffffffl;
	
	public static record DmabufFormat(int code, long modifier) {
	}
	
	public static class EGLError extends Exception {
		
		public EGLError() {
			this(EGL.eglGetError());
		}
		
		public EGLError(int code) {
			super(String.format("EGL Error %x", code));
		}
		
	}
	
}
