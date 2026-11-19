import { Stack } from '@astryxdesign/core/Stack';
import { Text } from '@astryxdesign/core/Text';
import { Badge } from '@astryxdesign/core/Badge';
import { Button } from '@astryxdesign/core/Button';
import { IconButton } from '@astryxdesign/core/IconButton';
import { Icon } from '@astryxdesign/core/Icon';
import { api } from '../lib/api';

const IMAGE_MIME = /^image\/(jpeg|png|gif|webp|svg\+xml)/i;

export function FileAttachment({ attachment, onDownload, onDelete, isImage }) {
  const isImg = isImage || IMAGE_MIME.test(attachment.mime || '');
  const isDownloadable = attachment.mime && !IMAGE_MIME.test(attachment.mime);

  const handleDownload = async () => {
    try {
      const res = await fetch(`/api/issues/${attachment.issueId}/attachments/${attachment.id}/file`, {
        headers: { Authorization: `Bearer ${localStorage.getItem('boron_token')}` }
      });
      if (!res.ok) throw new Error('Download failed');
      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = attachment.filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(url);
      onDownload?.();
    } catch (err) {
      alert('Failed to download file');
    }
  };

  return (
    <Stack direction="horizontal" gap={2} padding={2} variant="muted" elevation="none">
      {isImg ? (
        <Stack direction="horizontal" gap={2} vAlign="center">
          <img
            src={`/api/issues/${attachment.issueId}/attachments/${attachment.id}/file`}
            alt={attachment.filename}
            style={{ maxWidth: 120, maxHeight: 100, borderRadius: 6, objectFit: 'cover', cursor: 'pointer' }}
            onClick={() => window.open(`/api/issues/${attachment.issueId}/attachments/${attachment.id}/file`, '_blank')}
          />
          <Stack gap={1}>
            <Text size="sm" fontWeight="semibold">{attachment.filename}</Text>
            <Text size="xs" type="supporting">{attachment.mime}</Text>
          </Stack>
        </Stack>
      ) : (
        <Stack direction="horizontal" gap={2} vAlign="center">
          <IconButton label={`Download ${attachment.filename}`} icon={<Icon icon="file" />} variant="ghost" size="sm" />
          <Stack gap={1}>
            <Text size="sm" fontWeight="semibold">{attachment.filename}</Text>
            <Text size="xs" type="supporting">{attachment.mime || 'file'} • {attachment.size ? (attachment.size / 1024).toFixed(1) + ' KB' : ''}</Text>
          </Stack>
        </Stack>
      )}
      <Stack direction="horizontal" gap={1} hAlign="end">
        <IconButton label="Download" icon={<Icon icon="download" />} variant="ghost" size="sm" onClick={handleDownload} />
        {onDelete && <IconButton label="Delete" icon={<Icon icon="trash" />} variant="destructive" size="sm" onClick={onDelete} />}
      </Stack>
    </Stack>
  );
}