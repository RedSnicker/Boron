# Boron Feature Implementation Checklist

## Performance & UX
- [ ] Fix website slowness and unresponsiveness
- [ ] Make reply box much bigger
- [ ] Add icons to right-click menu items
- [ ] Remove @ and # auto-complete (show on type instead)
- [ ] Remove scaling thing bottom right of message box
- [ ] Button hover highlight for account dropdown
- [ ] Make icons more rectangular/square (solid)
- [ ] Remove theme toggle and notifications from account settings

## Channels & Issues
- [ ] Add channel settings
- [ ] Add issue settings
- [ ] Public/private toggle when creating issues
- [ ] Force at least one person or "skip" for private issues
- [ ] Auto-suggest users when adding people to team
- [ ] Remove add people button at top right in channels and issues
- [ ] Channel name at top much smaller
- [ ] Pinned messages show only icon, not whole thing
- [ ] Attachments button at left center of message box
- [ ] Emojis grouped with gifs and stickers in one button
- [ ] Users tab on right side of chat (online/DND/inactive/offline)
- [ ] Voice chat channel type
- [ ] Categories inside teams
- [ ] Drag channels around for organization
- [ ] Kick users from team

## Messages & Media
- [ ] Image preview with scale/move (click image to preview)
- [ ] Edit option on message hover
- [ ] Delete on message hover (red, trash icon)
- [ ] Fix image persistence
- [ ] Fix database persistence to /data

## Users & Roles
- [ ] User card popup (Discord-like) when clicking name/pfp
- [ ] Role system with permissions
- [ ] Assign roles to users (individual + role-based)
- [ ] Role-based permissions per channel (manage channels, messages, reactions, etc.)
- [ ] Specific permissions for specific channels for roles or individual users

## Backend & DevOps
- [ ] Terminal commands: user ban/setadmin/info username
- [ ] Numerical channel and team IDs
- [ ] New URL format: /teamid/channelid
- [ ] Database persistence to /data

## Deployment
- [ ] Create todolist.md
- [ ] Check off items by deploying
- [ ] Push all changes to GitHub