package dev.evvie.waylandcraft.desktop;

public class RawDesktopEntry {
	public final String appId;
	public final String name;
	public final String exec;

	public RawDesktopEntry(String appId, String name, String genericName, String exec, boolean execTerminal, String comment, String[] keywords, String[] categories, boolean visible, String iconPath) {
		this.appId = appId;
		this.name = name;
		this.exec = exec;
	}
}
