import { useState, useEffect } from 'react';
import { Stack } from '@astryxdesign/core/Stack';
import { Heading } from '@astryxdesign/core/Heading';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { TextInput } from '@astryxdesign/core/TextInput';
import { Badge } from '@astryxdesign/core/Badge';
import { HStack } from '@astryxdesign/core/HStack';
import { StackItem } from '@astryxdesign/core/Stack';
import { api } from '../lib/api';

export function IssuesView({ channelId, user }) {
  const [issues, setIssues] = useState([]);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');

  useEffect(() => {
    loadIssues();
  }, [channelId]);

  const loadIssues = async () => {
    try {
      const res = await api.get(`/api/issues?channel_id=${channelId}`);
      setIssues(res.data || []);
    } catch (err) {
      console.error('Failed to load issues', err);
    }
  };

  const handleCreate = async (e) => {
    e.preventDefault();
    if (!title.trim()) return;
    try {
      await api.post('/api/issues', {
        channel_id: channelId,
        title,
        description,
      });
      setTitle('');
      setDescription('');
      loadIssues();
    } catch (err) {
      alert('Failed to create issue');
    }
  };

  const getStatusBadge = (status) => {
    const variants = { open: 'primary', in_progress: 'secondary', review: 'warning', done: 'success', closed: 'gray' };
    return <Badge variant={variants[status] || 'secondary'}>{status}</Badge>;
  };

  return (
    <Stack padding={0} height="100%" width="100%">
      <Heading level={2} padding={3}>Issues</Heading>
      
      <Stack padding={3} gap={3}>
        <HStack gap={2}>
          <StackItem size="fill">
            <TextInput
              label="Issue Title"
              placeholder="Enter issue title..."
              value={title}
              onChange={(v) => setTitle(v)}
            />
          </StackItem>
          <StackItem size="static">
            <Button label="Create" variant="primary" onClick={handleCreate} />
          </StackItem>
        </HStack>
        <TextInput
          label="Description"
          placeholder="Describe the issue..."
          value={description}
          onChange={(v) => setDescription(v)}
        />
      </Stack>
      
      <Stack padding={3} gap={2} height="calc(100% - 200px)" isScrollable>
        {issues.map(issue => (
          <Card key={issue.id} padding={2} elevation="low">
            <Stack direction="horizontal" gap={2} hAlign="between" paddingBlockEnd={1}>
              <Heading level={3} size="sm">{issue.title}</Heading>
              {getStatusBadge(issue.status)}
            </Stack>
            {issue.description && <Text size="sm" type="supporting">{issue.description}</Text>}
            <Stack direction="horizontal" gap={2} paddingBlockStart={2}>
              <Badge variant="secondary">{issue.priority}</Badge>
              <Text size="xs" type="supporting">Reported: {new Date(issue.createdAt).toLocaleDateString()}</Text>
            </Stack>
          </Card>
        ))}
      </Stack>
    </Stack>
  );
}