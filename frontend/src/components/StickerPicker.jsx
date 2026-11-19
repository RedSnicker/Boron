import { useState } from 'react';
import { Stack } from '@astryxdesign/core/Stack';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { TextInput } from '@astryxdesign/core/TextInput';

const DEFAULT_STICKERS = [
  { id: 's1', name: 'Thumbs Up', emoji: '👍', type: 'emoji' },
  { id: 's2', name: 'Fire', emoji: '🔥', type: 'emoji' },
  { id: 's3', name: 'Celebration', emoji: '🎉', type: 'emoji' },
  { id: 's4', name: 'Rocket', emoji: '🚀', type: 'emoji' },
  { id: 's5', name: 'Coffee', emoji: '☕', type: 'emoji' },
  { id: 's6', name: 'Pizza', emoji: '🍕', type: 'emoji' },
  { id: 's7', name: 'Star', emoji: '⭐', type: 'emoji' },
  { id: 's8', name: 'Bug', emoji: '🐛', type: 'emoji' },
  { id: 's9', name: 'Gift', emoji: '🎁', type: 'emoji' },
  { id: 's10', name: 'Heart', emoji: '❤️', type: 'emoji' },
];

export function StickerPicker({ onSelect, isOpen, onClose, customStickers = [] }) {
  const [search, setSearch] = useState('');
  const allStickers = [...DEFAULT_STICKERS, ...customStickers];
  const filtered = allStickers.filter(s => 
    search.toLowerCase().includes(s.name.toLowerCase()) || search === ''
  );

  if (!isOpen) return null;

  return (
    <Card padding={3} elevation="high" style={{ width: 340, position: 'absolute', zIndex: 1000 }}>
      <TextInput label="Search stickers" value={search} onChange={(v) => setSearch(v)} placeholder="Search stickers..." />
      <Stack padding={2} gap={1} isScrollable height={240}>
        <Stack direction="horizontal" gap={1} wrap="wrap">
          {filtered.map(sticker => (
            <Button
              key={sticker.id}
              label={sticker.emoji || sticker.name}
              variant="ghost"
              size="sm"
              style={{ minWidth: 34, height: 34, borderRadius: 8, cursor: 'pointer' }}
              onClick={() => { onSelect(sticker); onClose(); }}
            />
          ))}
        </Stack>
      </Stack>
      <Stack padding={2} gap={1} variant="muted">
        <Text size="xs" type="supporting">Custom Stickers</Text>
        <Button label="+ Add Custom Sticker" variant="secondary" size="sm" width="100%" onClick={() => {
          const name = window.prompt('Sticker name:');
          if (name) {
            const emoji = window.prompt('Emoji or image URL:');
            if (emoji) {
              onSelect({ id: `custom_${Date.now()}`, name, emoji, type: 'custom' });
              onClose();
            }
          }
        }} />
      </Stack>
    </Card>
  );
}