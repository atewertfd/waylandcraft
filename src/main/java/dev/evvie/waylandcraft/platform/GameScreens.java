package dev.evvie.waylandcraft.platform;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Screen;

/**
 * Minecraft 26.2 moved screen changes onto {@code Minecraft.gui}.
 */
public final class GameScreens {

	private GameScreens() {
	}

	public static void set(Minecraft minecraft, Screen screen) {
		minecraft.gui.setScreen(screen);
	}

	public static Screen current(Minecraft minecraft) {
		return minecraft.gui.screen();
	}

	public static void systemMessage(Minecraft minecraft, net.minecraft.network.chat.Component message) {
		minecraft.gui.chatListener().handleSystemMessage(message, false);
	}
}
