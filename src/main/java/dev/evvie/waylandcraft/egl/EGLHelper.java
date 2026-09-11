package dev.evvie.waylandcraft.egl;

import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;

import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryUtil;

import dev.evvie.waylandcraft.bridge.dmabuf.Dmabuf;
import dev.evvie.waylandcraft.bridge.dmabuf.DmabufFormat;
import dev.evvie.waylandcraft.bridge.dmabuf.DmabufPlane;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLDeviceEXT;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLDisplay;
import dev.evvie.waylandcraft.egl.EGLTypes.EGLImage;

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
	
	private static final long[] PLANE_FD_ATTR = {
			EGL.EGL_DMA_BUF_PLANE0_FD_EXT,
			EGL.EGL_DMA_BUF_PLANE1_FD_EXT,
			EGL.EGL_DMA_BUF_PLANE2_FD_EXT,
			EGL.EGL_DMA_BUF_PLANE3_FD_EXT,
	};
	private static final long[] PLANE_OFFSET_ATTR = {
			EGL.EGL_DMA_BUF_PLANE0_OFFSET_EXT,
			EGL.EGL_DMA_BUF_PLANE1_OFFSET_EXT,
			EGL.EGL_DMA_BUF_PLANE2_OFFSET_EXT,
			EGL.EGL_DMA_BUF_PLANE3_OFFSET_EXT,
	};
	private static final long[] PLANE_PITCH_ATTR = {
			EGL.EGL_DMA_BUF_PLANE0_PITCH_EXT,
			EGL.EGL_DMA_BUF_PLANE1_PITCH_EXT,
			EGL.EGL_DMA_BUF_PLANE2_PITCH_EXT,
			EGL.EGL_DMA_BUF_PLANE3_PITCH_EXT,
	};
	private static final long[] PLANE_MOD_LO_ATTR = {
			EGL.EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT,
			EGL.EGL_DMA_BUF_PLANE1_MODIFIER_LO_EXT,
			EGL.EGL_DMA_BUF_PLANE2_MODIFIER_LO_EXT,
			EGL.EGL_DMA_BUF_PLANE3_MODIFIER_LO_EXT,
	};
	private static final long[] PLANE_MOD_HI_ATTR = {
			EGL.EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT,
			EGL.EGL_DMA_BUF_PLANE1_MODIFIER_HI_EXT,
			EGL.EGL_DMA_BUF_PLANE2_MODIFIER_HI_EXT,
			EGL.EGL_DMA_BUF_PLANE3_MODIFIER_HI_EXT,
	};
	
	public static EGLImage importDmabuf(EGLDisplay dpy, Dmabuf dmabuf) {
		ArrayList<Long> attribs = new ArrayList<Long>();
		attribs.add(EGL.EGL_WIDTH); attribs.add(Integer.toUnsignedLong(dmabuf.width()));
		attribs.add(EGL.EGL_HEIGHT); attribs.add(Integer.toUnsignedLong(dmabuf.height()));
		attribs.add(EGL.EGL_LINUX_DRM_FOURCC_EXT); attribs.add(Integer.toUnsignedLong(dmabuf.format()));
		
		long modifier = dmabuf.modifier();
		boolean hasModifier = modifier != DRM_MODIFIER_INVALID && modifier != DRM_MODIFIER_LINEAR;
		
		long modLo = modifier & U32_MAX;
		long modHi = modifier >>> 32;
		
		DmabufPlane[] planes = dmabuf.planes();
		for(int i = 0; i < planes.length; i++) {
			DmabufPlane plane = planes[i];
			attribs.add(PLANE_FD_ATTR[i]); attribs.add(Integer.toUnsignedLong(plane.fd()));
			attribs.add(PLANE_OFFSET_ATTR[i]); attribs.add(Integer.toUnsignedLong(plane.offset()));
			attribs.add(PLANE_PITCH_ATTR[i]); attribs.add(Integer.toUnsignedLong(plane.stride()));
			
			if(hasModifier) {
				attribs.add(PLANE_MOD_LO_ATTR[i]); attribs.add(modLo);
				attribs.add(PLANE_MOD_HI_ATTR[i]); attribs.add(modHi);
			}
		}
		attribs.add(EGL.EGL_NONE);
		
		PointerBuffer attribBuf = MemoryUtil.memAllocPointer(attribs.size());
		for(long attr : attribs) {
			attribBuf.put(attr);
		}
		
		return EGL.eglCreateImage(dpy, EGL.EGL_NO_CONTEXT, (int) EGL.EGL_LINUX_DMA_BUF_EXT, null, attribBuf);
	}
	
	private static final long U32_MAX = 0xffffffffl;
	
	public static final long DRM_MODIFIER_INVALID = 0xffffffffffffffl;
	public static final long DRM_MODIFIER_LINEAR = 0;
	
	public static class EGLError extends Exception {
		
		public EGLError() {
			this(EGL.eglGetError());
		}
		
		public EGLError(int code) {
			super(String.format("EGL Error %x", code));
		}
		
	}
	
}
