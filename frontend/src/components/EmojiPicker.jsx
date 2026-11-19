import { useState } from 'react';
import { Stack } from '@astryxdesign/core/Stack';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { TextInput } from '@astryxdesign/core/TextInput';

const EMOJIS = [
  '😀','😁','😂','🤣','😃','😄','😅','😆','😉','😊','😋','😎','😍','😘','🥰','😗',
  '😙','😜','🤪','😝','🤗','🤭','🤫','🤔','😐','😑','😬','🙄','😯','😲','😳','🥴',
  '😞','😟','😠','😡','😷','🤒','🤕','🤢','🤮','🤧','😇','😈','👿','👹','👺','🤡',
  '👽','🤖','💩','🙂','🤠','🥳','😻','😼','😽','🙀','😿','😹','😸','👍','👎','👌',
  '👊','✊','🤛','🤜','👋','🤚','🖐','✋','🖖','👌','✌','🤞','🤟','🤘','🤙','👈',
  '👉','👆','👇','☝','👍','👎','✊','🤛','🤜','👋','🤚','🖐','✋','🖖','👌','✌',
  '🤞','🤟','🤘','🤙','💪','🦾','🦵','🦶','👂','🦻','👃','🧠','🦷','🦴','👀','👁',
  '👅','👄','💋','🌹','🥀','🌺','🌸','🌼','🌷','🌻','🌞','🌝','🌛','🌜','🌚','🌞',
  '🌟','🌠','🌌','🌎','🌍','🌏','🌎','🪐','🌑','🌒','🌓','🌔','🌕','🌖','🌗','🌘',
  '🔥','💥','💫','💦','💨','🌪','🌧','☔','❄','☃','☀','🌈','⚡','☄','🔭','🔬',
  '📱','💻','🖥','🖨','⌨','🖱','🖲','💽','💾','💿','📀','📼','📷','📸','📹','📽',
  '🎥','📞','☎','📟','📠','📡','🔋','🔌','💡','🔦','🕯','🗑','🛢','🔧','🔨','⚒',
  '🛠','🗡','⚔','🔫','🏹','🛡','🚬','⚰','⚱','🏺','🧧','🍷','🍸','🍹','🍺','🍻',
  '🥂','🥃','🥤','🍶','🍵','☕','🍱','🍛','🍜','🍝','🍠','🍢','🍣','🍤','🍥','🍦',
  '🍧','🍨','🍩','🍪','🎂','🍴','🍽','🥄','🍴','🍵','🍶','🍺','🍻','🥂','🥃'
];

export function EmojiPicker({ onSelect, isOpen, onClose }) {
  const [search, setSearch] = useState('');

  const filtered = EMOJIS.filter(e => search.toLowerCase().includes(e.toLowerCase()) || search === '');

  if (!isOpen) return null;

  return (
    <Card padding={3} elevation="high" style={{ width: 320, position: 'absolute', zIndex: 1000 }}>
      <TextInput label="Search emojis" value={search} onChange={(v) => setSearch(v)} placeholder="Type an emoji..." />
      <Stack padding={2} gap={1} isScrollable height={240}>
        <Stack direction="horizontal" gap={1} wrap="wrap">
          {filtered.map(emoji => (
            <Button
              key={emoji}
              label={emoji}
              variant="ghost"
              size="sm"
              style={{ minWidth: 32, height: 32, borderRadius: 8, cursor: 'pointer' }}
              onClick={() => { onSelect(emoji); onClose(); }}
            />
          ))}
        </Stack>
      </Stack>
    </Card>
  );
}