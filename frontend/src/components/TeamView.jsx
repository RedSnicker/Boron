import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Stack } from '@astryxdesign/core/Stack';
import { Heading } from '@astryxdesign/core/Heading';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { Badge } from '@astryxdesign/core/Badge';
import { ChatView } from './ChatView';
import { IssuesView } from './IssuesView';
import { NotesView } from './NotesView';

export function TeamView({ user }) {
  const navigate = useNavigate();
  const { teamId } = useParams();
  const [activeTab, setActiveTab] = useState('chat');

  const tabs = [
    { key: 'chat', label: 'Chat', component: <ChatView channelId={teamId} user={user} /> },
    { key: 'issues', label: 'Issues', component: <IssuesView channelId={teamId} user={user} /> },
    { key: 'notes', label: 'Sticky Notes', component: <NotesView channelId={teamId} user={user} /> },
  ];

  return (
    <Stack padding={0} height="100vh" width="100%">
      <Stack direction="horizontal" padding={3} hAlign="between" vAlign="center" elevation="low">
        <Stack direction="horizontal" gap={2} vAlign="center">
          <Button label="← Back" variant="ghost" size="sm" as="a" href="/" />
          <Heading level={1} size="lg">Team Workspace</Heading>
        </Stack>
        <Stack direction="horizontal" gap={2} vAlign="center">
          <Badge variant="secondary">{user.role}</Badge>
          <Stack direction="horizontal" gap={1}>
            {tabs.map(tab => (
              <Button
                key={tab.key}
                label={tab.label}
                variant={activeTab === tab.key ? 'primary' : 'ghost'}
                size="sm"
                onClick={() => setActiveTab(tab.key)}
              />
            ))}
          </Stack>
          <Button label="Settings" variant="ghost" size="sm" />
        </Stack>
      </Stack>
      
      <Stack padding={4} gap={4} height="calc(100vh - 72px)">
        {tabs.find(t => t.key === activeTab)?.component}
      </Stack>
    </Stack>
  );
}