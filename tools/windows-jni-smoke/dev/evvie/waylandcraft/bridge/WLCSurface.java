package dev.evvie.waylandcraft.bridge;

public class WLCSurface {
	private long handle;
	protected boolean visited;
	protected WLCSurface nextChild;
	protected WLCSurface prevChild;
	protected long parentHandle;
	protected int xoff;
	protected int yoff;
	int width;
	int height;
	int stride;
	int format;
	long ptr;
	int attachCount;

	protected WLCSurface(long handle) {
		this.handle = handle;
	}

	protected void removeBuffer() {}

	protected void setViewportSrc(double x, double y, double width, double height) {}

	protected void setViewportDst(int width, int height) {
		this.width = width;
		this.height = height;
	}

	protected void attachShmBuffer(long ptr, int width, int height, int format, int stride) {
		this.ptr = ptr;
		this.width = width;
		this.height = height;
		this.format = format;
		this.stride = stride;
		this.attachCount++;
	}

	protected void attachSinglePixelBuffer(byte r, byte g, byte b, byte a) {}

	protected boolean attachDmabuf(long handle) {
		return false;
	}

	protected void attachNewDmabuf(long handle, long eglImage, int width, int height) {}

	protected void clearDamage() {}

	protected void addBufferDamage(int x, int y, int width, int height) {}

	protected void addSurfaceDamage(int x, int y, int width, int height) {}
}
