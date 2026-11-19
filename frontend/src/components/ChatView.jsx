import { useState, useEffect } from 'react';
import { Stack } from '@astryxdesign/core/Stack';
import { Heading } from '@astryxdesign/core/Heading';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { TextInput } from '@astryxdesign/core/TextInput';
import { HStack } from '@astryxdesign/core/HStack';
import { StackItem } from '@astryxdesign/core/Stack';
import { api } from '../lib/api';

export function ChatView({ channelId, user }) {
  const [messages, setMessages] = useState([]);
  const [content, setContent] = useState('');

  useEffect(() => {
    loadMessages();
  }, [channelId]);

  const loadMessages = async () => {
    try {
      const res = await api.get(`/api/channels/${channelId}/messages`);
      setMessages(res.data || []);
    } catch (err) {
      console.error('Failed to load messages', err);
    }
  };

  const handleSend = async (e) => {
    e.preventDefault();
    if (!content.trim()) return;
    try {
      await api.post(`/api/channels/${channelId}/messages`, { channel_id: channelId, content });
      setContent('');
      loadMessages();
    } catch (err) {
      alert('Failed to send message');
    }
  };

  return (
    <Stack padding={0} height="100%" width="100%">
      <Heading level={2} padding={3}>Team Chat</Heading>
      <Stack gap={0} height="calc(100% - 120px)" isScrollable padding={3}>
        {messages.map(msg => (
          <Card key={msg.id} padding={2} marginBottom={2}>
            <Stack direction="horizontal" gap={2} hAlign="between" paddingBlockEnd={1}>
              <Text size="sm" fontWeight="semibold">{msg.displayName}</Text>
              <Text size="xs" type="supporting">{new Date(msg.createdAt).toLocaleTimeString()}</Text>
            </Stack>
            <Text>{msg.content}</Text>
            {msg.githubCommitSha && (
              <Stack direction="horizontal" gap={1} paddingBlockStart={2}>
                <Badge variant="secondary">GitHub Commit</Badge>
                <Text size="xs" type="supporting">{msg.githubCommitSha}</Text>
              </Stack>
            )}
            {msg.githubPrNumber && (
              <Stack direction="horizontal" gap={1} paddingBlockStart={2}>
                <Badge variant="primary">GitHub PR #{msg.githubPrNumber}</Badge>
              </Stack>
            )}
          </Card>
        ))}
      </Stack>
      <Stack padding={3} hAlign="stretch">
        <HStack gap={2}>
          <StackItem size="fill">
            <TextInput
              label="Message"
              isLabelHidden
              placeholder="Type a message..."
              value={content}
              onChange={(v) => setContent(v)}
              enterKeyHint="send"
            />
          </StackItem>
          <StackItem size="static">
            <Button label="Send" variant="primary" onClick={handleSend} />
          </StackItem>
        </HStack>
      </Stack>
    </Stack>
  );
}