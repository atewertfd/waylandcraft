package dev.evvie.waylandcraft.platform;

import dev.evvie.waylandcraft.WaylandCraftCommon;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Screen;

/**
 * Minecraft 26.2 moved screen changes onto {@code Minecraft.gui}.
 */
public final class GameScreens {

	private GameScreens() {
	}

	public static void set(Minecraft minecraft, Screen screen) {
		WaylandCraftCommon.LOGGER.info("Opening screen {}", screen == null ? "null" : screen.getClass().getSimpleName());
		minecraft.gui.setScreen(screen);
	}

	public static Screen current(Minecraft minecraft) {
		return minecraft.gui.screen();
	}

	public static void systemMessage(Minecraft minecraft, net.minecraft.network.chat.Component message) {
		minecraft.gui.chatListener().handleSystemMessage(message, false);
	}
}
