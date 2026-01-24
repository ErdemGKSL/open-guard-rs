module-config-name = Configuration
module-config-desc = Bot settings and module configuration.

module-channel-protection-name = Channel Protection
module-channel-protection-desc = Prevents unauthorized channel creation, deletion, or updates.

module-channel-permission-protection-name = Channel Permission Protection
module-channel-permission-protection-desc = Protect channel permission overwrites from unauthorized changes.

module-role-protection-name = Role Protection
module-role-protection-desc = Protect roles from unauthorized creation or deletion.

module-role-permission-protection-name = Role Permission Protection
module-role-permission-protection-desc = Protect role permissions from unauthorized changes.

module-member-permission-protection-name = Member Permission Protection
module-member-permission-protection-desc = Protect members from being granted dangerous permissions via roles.

module-bot-adding-protection-name = Bot Adding Protection
module-moderation-protection-name = Moderation Protection
module-bot-adding-protection-desc = Automatically kicks newly added bots and punishes the user who added them.
module-moderation-protection-desc = Logs and limits moderation actions by authorized users. Commands bypass these limits.
module-logging-name = Logging
module-logging-label = Logging
module-logging-desc = Comprehensive server event logging (Messages, Voice, Members)

module-sticky-roles-name = Sticky Roles
module-sticky-roles-desc = Restores member roles when they rejoin the server.

help-title = Bot Help Menu

config-general-header = ⚙️ **General Configuration**
config-log-channel-label = **General Log Channel**
config-jail-role-label = **Jail Role**
config-modules-header = 📦 **Modules**
config-select-log-channel-placeholder = Select general log channel...
config-select-module-log-channel-placeholder = Select log channel for this module...
config-module-log-channel-label = **Module Log Channel**
config-select-jail-role-placeholder = Select jail role...
config-select-module-placeholder = Select a module to configure...
config-log-field-user = User
config-log-field-channel = Channel
config-success-update = ✅ Configuration updated successfully!
config-btn-enabled = Enabled
config-btn-disabled = Disabled
config-status-enabled = Enabled
config-status-disabled = Disabled
config-error-update = ❌ Failed to update configuration.
config-channel-protection-label = Channel Protection
config-channel-protection-desc = Configure channel creation/deletion protection
config-channel-permission-protection-label = Channel Permission Protection
config-channel-permission-protection-desc = Configure channel permission overwrite protection
config-role-protection-label = Role Protection
config-role-protection-desc = Configure role creation/deletion protection
config-role-permission-protection-label = Role Permission Protection
config-role-permission-protection-desc = Configure role permission protection

config-member-permission-protection-label = Member Permission Protection
config-member-permission-protection-desc = Configure member role/permission protection

config-bot-adding-protection-label = Bot Adding Protection
config-bot-adding-protection-desc = Configure bot adding protection
config-moderation-protection-label = Moderation Protection
config-moderation-protection-desc = Logs and limits moderation actions by authorized users
config-logging-label = Logging
config-logging-desc = Comprehensive server event logging (Messages, Voice, Members)
config-sticky-roles-label = Sticky Roles
config-sticky-roles-desc = Restores member roles when they rejoin the server
config-punishment-label = Punishment
config-select-punishment-placeholder = Select punishment...
config-punishment-type-none = None
config-punishment-type-unperm = Unperm
config-punishment-type-ban = Ban
config-punishment-type-kick = Kick
config-punishment-type-jail = Jail
config-revert-label = Revert unauthorized actions
config-repetition-at-label = At Repetition: {$count}
config-repetition-interval-label = Interval: {$count} min
config-back-label = Back
config-cp-ignore-private-label = Ignore Private Channels
config-cp-punish-create = Create
config-cp-punish-update = Update
config-cp-punish-delete = Delete
config-cp-punish-when-placeholder = When to punish?
config-cpp-ignore-private-label = Ignore Private Channels
config-cpp-punish-create = Create
config-cpp-punish-update = Update
config-cpp-punish-delete = Delete
config-cpp-punish-when-placeholder = When to punish?

config-rp-punish-create = Create
config-rp-punish-update = Update
config-rp-punish-delete = Delete
config-rp-punish-when-placeholder = When to punish?

config-rpp-punish-update = Update
config-rpp-punish-when-placeholder = When to punish?

config-bap-punish-add = Bot Added
config-bap-punish-when-placeholder = When to punish?

config-whitelists-btn = Whitelists
config-whitelists-view-btn = View
config-whitelist-manage-btn = Manage
config-whitelist-delete-btn = Delete
config-whitelist-next-page = Next
config-whitelist-prev-page = Previous
config-whitelist-manage-title = **Manage Whitelist Entry**
config-whitelist-selector-header = **Whitelists Configuration**
config-whitelist-global-btn = Global Whitelists
config-whitelist-select-module-placeholder = Select a module...
config-whitelist-global-header = **Global Whitelists**
config-whitelist-channel-protection-header = **Channel Protection Whitelists**
config-whitelist-channel-permission-protection-header = **Channel Permission Protection Whitelists**
config-whitelist-role-protection-header = **Role Protection Whitelists**
config-whitelist-role-permission-protection-header = **Role Permission Protection Whitelists**
config-whitelist-member-permission-protection-header = **Member Permission Protection Whitelists**
config-whitelist-bot-adding-protection-header = **Bot Adding Protection Whitelists**
config-whitelist-moderation-protection-header = **Moderation Protection Whitelists**
config-whitelist-logging-header = **Logging Whitelists**
config-whitelist-sticky-roles-header = **Sticky Roles Whitelists**
config-whitelist-users-label = **Whitelisted Users**
config-whitelist-delete-user-placeholder = Select user to remove...
config-whitelist-roles-label = **Whitelisted Roles**
config-whitelist-delete-role-placeholder = Select role to remove...
config-whitelist-add-user-btn = Add User
config-whitelist-add-role-btn = Add Role
config-select-user-placeholder = Select a user...
config-select-role-placeholder = Select a role...
config-select-level-placeholder = Select whitelist level...
config-level-head = Head
config-level-admin = Admin
config-level-invulnerable = Invulnerable

help-page-0-title = 🚀 Getting Started
help-page-0-content = 
    ### Introduction
    Open Guard is a high-performance bot designed to protect your server from unauthorized changes. It monitors audit logs in real-time to revert malicious actions and punish offenders.
    
    ### Configuration
    Use the ` /config ` command to manage the bot. From there, you can:
    - Enable or disable specific protection modules.
    - Set dedicated log channels for each module.
    - Configure punishment types and thresholds.

help-page-1-title = ⚖️ Punishment & Violation Logic
help-page-1-content = 
    ### Violations
    Each unauthorized action increases a user's violation counter for that specific module. If they repeat the action within the configured interval, they will be punished.
    
    ### Punishments
    - **Unperm**: Removes roles containing dangerous permissions (e.g., Administrator, Manage Roles). It safely skips managed roles and roles positioned above the bot.
    - **Ban / Kick**: Immediately removes the user from the server.
    - **Jail**: Assigns a configured 'Jail' role to the offender.

help-page-2-title = 🛡️ Whitelist Logic
help-page-2-content = 
    ### Implicit Whitelist (Hierarchy)
    Trust is automatically granted based on server hierarchy:
    - **Head**: Server Owner (Full immunity).
    - **Admin**: Users with Admin permission positioned above the bot.
    - **Invulnerable**: Users positioned above the bot.
    
    ### Explicit Whitelist
    You can whitelist specific users or roles using the ` /config ` menus. These can be global or limited to specific modules.

help-page-3-title = 🔑 Permission Requirements
help-page-3-content = 
    ### Bot Permissions
    To function effectively, Open Guard requires the following permissions:
    - **View Audit Log**: Essential for detecting who performed an action.
    - **Manage Roles**: Needed to revert role changes and apply Unperm/Jail.
    - **Manage Channels**: Needed to revert channel deletions or edits.
    - **Extensions**: Kick and Ban permissions for respective punishments.

help-prev-btn = Previous
help-next-btn = Next

# Logging
log-status-whitelisted = ✅ **Whitelisted ({$level})**\n> Unauthorized action detected but ignored due to whitelist level.
log-status-unauthorized = 🚨 **Unauthorized Action Detected**\n> Applying protection and punishment...
log-status-punished = 🚨 **Blocked & Punished** ({$type})\n> Action was unauthorized. User has been punished.
log-status-violation = 🚨 **Violation Recorded** ({$current}/{$threshold})\n> Action was unauthorized. Punishment will trigger at {$threshold} violations.
log-status-blocked = 🚨 **Blocked** (No Punishment)\n> Action was unauthorized, but no specific punishment is configured for this module.
log-status-reverted = \n✅ **Successfully Reverted**\n> The unauthorized changes have been rolled back.
log-status-revert-failed = \n❌ **Revert Failed**\n> Failed to roll back changes. Check bot permissions.
log-status-skipped = \n🛡️ **Protection Skipped**\n> User has **{$level}** level permissions, bypassing **{$punishment}** punishment.
log-status-no-revert = \nℹ️ **Nothing to Revert**\n> No revertible changes were found in the audit log entry.
log-status-not-enabled = ℹ️ **Protection not enabled**\n> This protection is currently disabled in the module configuration.

log-field-acting-user = Acting User
log-field-target-member = Target Member
log-field-added-perms = Added Perms
log-field-action-status = Status
log-field-role = Role
log-field-channel = Channel
log-field-user = User
log-field-reason = Reason
log-field-duration = Duration
log-field-type = Type
log-val-permanent = Permanent
log-val-no-reason = No reason provided
log-val-temp-ban-expired = Temporary ban expired

log-member-perm-title-whitelisted = Permission Sharing (Whitelisted)
log-member-perm-title-blocked = Permission Sharing (Blocked)
log-member-perm-desc = <@{$userId}> added dangerous permissions to <@{$targetId}> via roles.
log-member-perm-reason-update = Unauthorized Permission Sharing
log-member-perm-revert-reason = Unauthorized Permission Sharing Revert

log-chan-title-whitelisted = Channel Modified (Whitelisted)
log-chan-title-blocked = Channel Modified (Blocked)
log-chan-title-logged = Channel Modified (Logged)
log-chan-desc-create = A new channel (<#{$channelId}>) was created by <@{$userId}>.
log-chan-desc-delete = A channel (`{$channelId}`) was deleted by <@{$userId}>.
log-chan-desc-update = A channel (<#{$channelId}>) was modified by <@{$userId}>.
log-chan-reason-create = Channel Created
log-chan-reason-delete = Channel Deleted
log-chan-reason-update = Channel Updated
log-chan-revert-reason = Channel Protection Revert

log-role-title-whitelisted = Role Modified (Whitelisted)
log-role-title-blocked = Role Modified (Blocked)
log-role-title-logged = Role Modified (Logged)
log-role-desc-create = A new role (`{$roleId}`) was created by <@{$userId}>.
log-role-desc-delete = A role (`{$roleId}`) was deleted by <@{$userId}>.
log-role-desc-update = A role (<@&{$roleId}>) was modified by <@{$userId}>.
log-role-reason-create = Role Created
log-role-reason-delete = Role Deleted
log-role-reason-update = Role Updated
log-role-revert-reason = Role Protection Revert

log-chan-perm-title-whitelisted = Permission Overwrite Modified (Whitelisted)
log-chan-perm-title-blocked = Permission Overwrite Modified (Blocked)
log-chan-perm-title-logged = Permission Overwrite Modified (Logged)
log-chan-perm-desc-create = A permission overwrite in channel (<#{$channelId}>) was created by <@{$userId}>.
log-chan-perm-desc-delete = A permission overwrite in channel (<#{$channelId}>) was deleted by <@{$userId}>.
log-chan-perm-desc-update = A permission overwrite in channel (<#{$channelId}>) was modified by <@{$userId}>.
log-chan-perm-reason-create = Channel Permission Overwrite Created
log-chan-perm-reason-delete = Channel Permission Overwrite Deleted
log-chan-perm-reason-update = Channel Permission Overwrite Updated
log-chan-perm-revert-reason = Channel Permission Protection Revert

log-role-perm-title-whitelisted = Role Permission Modified (Whitelisted)
log-role-perm-title-blocked = Role Permission Modified (Blocked)
log-role-perm-title-logged = Role Permission Modified (Logged)
log-role-perm-desc-update = Permissions for role <@&{$roleId}> were modified by <@{$userId}>.
log-role-perm-reason-update = Role Permissions Updated
log-role-permission-revert-reason = Role Permission Protection Revert

log-bot-add-title-whitelisted = Bot Added (Whitelisted)
log-bot-add-title-blocked = Bot Added (Blocked)
log-bot-add-desc = Bot <@{$botId}> (`{$botId}`) was added by <@{$userId}>.
log-bot-add-reason = Unauthorized Bot Added
log-bot-add-revert-reason = Bot Adding Protection Revert

status-title = 📊 Bot Status
status-modules = 📦 Enabled Modules
status-latency = ⏱️ Latency
status-shard = 💎 Shard
status-system = 🖥️ System Resources
status-no-guild = ❌ Not in a guild.
status-refresh-btn = Refresh
status-metrics = Metrics

mod-error-invalid-duration = Invalid duration format! Use something like 1d, 1h, or 10m30s.
mod-ban-success-temp = ✅ Banned <@{$userId}> for {$duration} (Reason: {$reason})
mod-ban-success-perm = ✅ Banned <@{$userId}> permanently (Reason: {$reason})
mod-kick-success = ✅ Kicked <@{$userId}> (Reason: {$reason})
mod-timeout-success = ✅ Timed out <@{$userId}> for {$duration} (Reason: {$reason})
mod-jail-success-temp = ✅ Jailed <@{$userId}> for {$duration} (Reason: {$reason})
mod-jail-success-perm = ✅ Jailed <@{$userId}> permanently (Reason: {$reason})
mod-unjail-success = ✅ Unjailed <@{$userId}>

mod-warn-remaining-2 = ⚠️ **Moderation Limit Warning**\nYou have last **2** moderation actions to execute.
mod-warn-remaining-1 = ⚠️ **Moderation Limit Warning**\nYou have last **1** moderation action to execute.
mod-warn-limit-reached = 🛑 **Moderation Limit Reached**\nYou don't have any more moderation actions that you can execute. Please don't do that or I will punish you! (Next punishment: **{$punishment}**)

# Moderation Logging
log-mod-jail-title = User Jailed
log-mod-jail-desc = User <@{$userId}> has been jailed.
log-mod-unjail-title = User Unjailed
log-mod-unjail-desc = User <@{$userId}> has been unjailed.
log-mod-unban-title = User Unbanned
log-mod-unban-desc = User <@{$userId}> has been unbanned (temporary ban expired).
log-mod-punish-title = Automated Punishment Applied
log-mod-punish-desc = User <@{$userId}> has been automatically punished.

log-mod-jail-cmd-title = Jail Command Executed
log-mod-jail-cmd-desc = Moderator <@{$modId}> jailed <@{$userId}>
log-mod-unjail-cmd-title = Unjail Command Executed
log-mod-unjail-cmd-desc = Moderator <@{$modId}> unjailed <@{$userId}>
log-mod-ban-cmd-title = Ban Command Executed
log-mod-ban-cmd-desc = Moderator <@{$modId}> banned <@{$userId}>
log-mod-kick-cmd-title = Kick Command Executed
log-mod-kick-cmd-desc = Moderator <@{$modId}> kicked <@{$userId}>
log-mod-timeout-cmd-title = Timeout Command Executed
log-mod-timeout-cmd-desc = Moderator <@{$modId}> timed out <@{$userId}>

log-mod-audit-title-whitelisted = Moderation Action (Whitelisted: {$action})
log-mod-audit-title-limited = Moderation Action (Limited: {$action})
log-mod-audit-title-logged = Moderation Action (Logged: {$action})
log-mod-audit-desc = User <@{$userId}> performed a `{$action}` on <@{$targetId}>.

# Logging Module Config
config-log-msg-label = Message Logging
config-log-voice-label = Voice Logging
config-log-member-label = Member Logging
config-log-channels-header = Per-type Log Channels (Optional)
config-log-msg-channel-placeholder = Select message log channel...
config-log-voice-channel-placeholder = Select voice log channel...
config-log-member-channel-placeholder = Select member log channel...
config-log-toggles-header = **Log Toggles**
config-page-general = General
config-page-channels = Channels
config-log-msg-channel-label = **Message Log Channel**
config-log-voice-channel-label = **Voice Log Channel**
config-log-member-channel-label = **Member Log Channel**

# Logging Events
log-msg-delete-title = Message Deleted
log-msg-delete-desc = { $userId ->
    [0] A message was deleted in <#{$channelId}>
    *[other] A message by <@{$userId}> was deleted in <#{$channelId}>
}
log-msg-delete-content = Content
log-msg-edit-title = Message Edited
log-msg-edit-desc = <@{$userId}> edited a message in <#{$channelId}>
log-msg-edit-before = Before
log-msg-edit-after = After
log-voice-join-title = Voice Join
log-voice-join-desc = <@{$userId}> joined <#{$channelId}>
log-voice-leave-title = Voice Leave
log-voice-leave-desc = <@{$userId}> left <#{$channelId}>
log-voice-move-title = Voice Move
log-voice-move-desc = <@{$userId}> moved from <#{$oldChannelId}> to <#{$newChannelId}>
log-voice-state-title = Voice State Change
log-voice-state-desc = <@{$userId}> changed state: {$state}
log-member-join-title = Member Joined
log-member-join-desc = <@{$userId}> joined the server
log-member-leave-title = Member Left
log-member-leave-desc = <@{$userId}> left the server
log-member-leave-roles = Roles

# Whitelist Modals
config-whitelist-modal-user-title-new = Add Whitelisted User
config-whitelist-modal-user-title-edit = Edit Whitelisted User
config-whitelist-modal-role-title-new = Add Whitelisted Role
config-whitelist-modal-role-title-edit = Edit Whitelisted Role
config-whitelist-modal-user-label = User
config-whitelist-modal-user-description = Select the user to whitelist
config-whitelist-modal-role-label = Role
config-whitelist-modal-role-description = Select the role to whitelist
config-whitelist-modal-level-label = Permission Level
config-whitelist-modal-level-description = Select the whitelist permission level
config-whitelist-modal-level-head = Head (Full Immunity)
config-whitelist-modal-level-admin = Admin (Bypass Punishment)
config-whitelist-modal-level-invulnerable = Invulnerable (Cannot be Punished)

# Setup
setup-cancelled = ❌ Setup cancelled.
setup-apply-success = ✅ **Setup applied successfully!** Your bot is now configured and ready to protect your server.
setup-step1-title = ## 🛠️ Step 1: System Selection
setup-step1-desc = Which systems do you want to enable? Unselected systems will be disabled.
setup-systems-placeholder = Select systems to enable...
setup-step2-title = ## 📜 Step 2: Fallback Logging
setup-step2-desc = Select a channel where system logs will be sent if a specific channel is not configured for a module. This is optional.
setup-logging-placeholder = Select fallback log channel (Optional)...
setup-logging-skip = Skip / No Fallback
setup-step3-title = ## 🛡️ Step 3: Global Whitelist
setup-step3-desc = Select users and roles that should be globally whitelisted from all protection systems. You can skip this by just clicking Next.
setup-whitelist-users-placeholder = Whitelist Users (Optional)...
setup-whitelist-roles-placeholder = Whitelist Roles (Optional)...
setup-next = Next
setup-step4-title = ## 🔧 Step 4: Configuring {$label}
setup-step4-generic-desc = This module doesn't require extra configuration in the fast setup. Click Next to continue.
setup-step4-logging-desc = Which types of activity do you want to log?
setup-logging-types-placeholder = Select log types to enable...
setup-log-type-messages = Messages
setup-log-type-messages-desc = Log message edits and deletions
setup-log-type-voice = Voice
setup-log-type-voice-desc = Log voice channel joins and leaves
setup-log-type-membership = Membership
setup-log-type-membership-desc = Log member joins and leaves
setup-summary-title = ## ✅ Final Step: Summary
setup-summary-desc = Review your changes and click Apply to save them to the database.
setup-summary-enabled-modules = **Enabled Modules:**
setup-summary-none = None (All modules will be disabled)
setup-summary-fallback-log = **Fallback Log Channel:**
setup-summary-whitelist = **Whitelist:**
setup-summary-users = { $count ->
    [one] 1 User
    *[other] {$count} Users
}
setup-summary-roles = { $count ->
    [one] 1 Role
    *[other] {$count} Roles
}
setup-apply = Apply Changes
setup-cancel = Cancel
setup-create-channel = Create Channel
setup-create-channel-fallback-name = bot-logs
setup-module-log-channel-title = **Log Channel for {$label}**
setup-module-log-channel-desc = Select or create a dedicated log channel for this module.
setup-module-log-channel-placeholder = Select log channel (Optional)...
setup-or-create = Or create a new channel:

# Channel Protection Setup
setup-cp-desc = Configure when channel protection should trigger and whether to ignore private channels.
setup-cp-punish-when-placeholder = Select which channel actions to protect against...
setup-cp-punish-create = Channel Create
setup-cp-punish-update = Channel Update
setup-cp-punish-delete = Channel Delete

# Channel Permission Protection Setup
setup-cpp-desc = Configure when channel permission protection should trigger and whether to ignore private channels.
setup-cpp-punish-when-placeholder = Select which permission actions to protect against...
setup-cpp-punish-create = Permission Create
setup-cpp-punish-update = Permission Update
setup-cpp-punish-delete = Permission Delete

# Role Protection Setup
setup-rp-desc = Configure when role protection should trigger.
setup-rp-punish-when-placeholder = Select which role actions to protect against...
setup-rp-punish-create = Role Create
setup-rp-punish-update = Role Update
setup-rp-punish-delete = Role Delete

# Moderation Protection Setup
setup-mp-desc = Configure which moderation actions should trigger protection.
setup-mp-punish-when-placeholder = Select which moderation actions to protect against...
setup-mp-punish-ban = Ban
setup-mp-punish-kick = Kick
setup-mp-punish-timeout = Timeout

# Invite Tracking Module
module-invite-tracking-name = Invite Tracking
module-invite-tracking-desc = Track server invites and see who invited whom
config-invite-tracking-label = Invite Tracking
config-it-vanity-label = Track Vanity URL Joins
config-it-ignore-bots-label = Ignore Bot Invites
config-it-min-age-label = Minimum Account Age (Days): {$count}
config-it-fake-threshold-label = Fake Member Threshold (Hours): {$count}
config-it-leaderboard-limit-label = Leaderboard Limit: {$count}
config-whitelist-invite-tracking-header = Whitelist: Invite Tracking

# Invite Tracking Setup
setup-it-desc = Configure invite tracking settings. Track who invited whom and analyze server growth.
