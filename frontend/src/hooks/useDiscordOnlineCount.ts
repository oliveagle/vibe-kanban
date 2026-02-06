import { useQuery } from '@tanstack/react-query';

const DISCORD_GUILD_ID = '1423630976524877857';

// Check if Discord integration is enabled via environment variable
const isDiscordEnabled = import.meta.env.VITE_ENABLE_DISCORD === 'true';

async function fetchDiscordOnlineCount(): Promise<number | null> {
  if (!isDiscordEnabled) {
    return null;
  }

  try {
    const res = await fetch(
      `https://discord.com/api/guilds/${DISCORD_GUILD_ID}/widget.json`,
      { cache: 'no-store' }
    );

    if (!res.ok) {
      console.warn(`Discord API error: ${res.status}`);
      return null;
    }

    const data = await res.json();
    if (typeof data?.presence_count === 'number') {
      return data.presence_count;
    }

    return null;
  } catch (error) {
    console.warn('Failed to fetch Discord online count:', error);
    return null;
  }
}

export function useDiscordOnlineCount() {
  return useQuery({
    queryKey: ['discord-online-count'],
    queryFn: fetchDiscordOnlineCount,
    refetchInterval: 10 * 60 * 1000,
    staleTime: 10 * 60 * 1000,
    retry: false,
    refetchOnMount: false,
    placeholderData: (previousData) => previousData,
    // Disable the query if Discord is not enabled
    enabled: isDiscordEnabled,
  });
}
