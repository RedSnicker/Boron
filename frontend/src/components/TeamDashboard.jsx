import { useState, useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Stack } from '@astryxdesign/core/Stack';
import { Heading } from '@astryxdesign/core/Heading';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { Grid } from '@astryxdesign/core/Grid';
import { Badge } from '@astryxdesign/core/Badge';
import { api } from '../lib/api';

export function TeamDashboard({ user }) {
  const navigate = useNavigate();
  const { teamId } = useParams();
  const [teams, setTeams] = useState([]);

  useEffect(() => {
    api.get('/api/teams')
      .then(res => setTeams(res.data || []))
      .catch(() => {});
  }, []);

  const handleCreateTeam = async () => {
    const name = prompt('Team name:');
    const slug = prompt('Team slug (url-friendly):');
    if (!name || !slug) return;
    try {
      const res = await api.post('/api/teams', { name, slug });
      navigate(`/team/${res.data.id}`);
    } catch (err) {
      alert('Failed to create team');
    }
  };

  return (
    <Stack padding={0} height="100vh" width="100%">
      <Stack direction="horizontal" padding={3} hAlign="between" vAlign="center" elevation="low">
        <Stack direction="horizontal" gap={2} vAlign="center">
          <Heading level={1} size="lg">Boron</Heading>
          <Badge variant="secondary">Team Workspace</Badge>
        </Stack>
        <Stack direction="horizontal" gap={2} vAlign="center">
          <Text size="sm">{user.displayName} ({user.role})</Text>
          <Button label="Sign out" variant="ghost" onClick={() => {
            localStorage.removeItem('boron_token');
            localStorage.removeItem('boron_user');
            window.location.href = '/login';
          }} />
        </Stack>
      </Stack>
      
      <Stack padding={4} gap={4}>
        <Stack direction="horizontal" gap={2} hAlign="between" vAlign="center">
          <Heading level={2}>Your Teams</Heading>
          <Button label="+ New Team" variant="primary" onClick={handleCreateTeam} />
        </Stack>
        
        {teams.length === 0 ? (
          <Card padding={4} hAlign="center" vAlign="center">
            <Text>No teams yet. Create your first team to start collaborating.</Text>
          </Card>
        ) : (
          <Grid columns={3} gap={3}>
            {teams.map(team => (
              <Card key={team.id} onClick={() => navigate(`/team/${team.id}`)} elevation="low">
                <Stack gap={2} paddingBlockEnd={2}>
                  <Heading level={3}>{team.name}</Heading>
                  <Text size="sm" type="supporting">{team.slug}</Text>
                  <Text size="sm" type="supporting">{team.memberCount} members</Text>
                </Stack>
              </Card>
            ))}
          </Grid>
        )}
      </Stack>
    </Stack>
  );
}