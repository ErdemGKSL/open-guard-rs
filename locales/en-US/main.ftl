module-config-name = Configuration
module-config-description = Bot settings and module configuration.

module-channel-protection-name = Channel Protection
module-channel-protection-description = Protect channels from unauthorized creation or deletion.

module-channel-permission-protection-desc = Protect channel permission overwrites from unauthorized changes.

module-role-protection-name = Role Protection
module-role-protection-description = Protect roles from unauthorized creation or deletion.

module-role-permission-protection-name = Role Permission Protection
module-role-permission-protection-desc = Protect role permissions from unauthorized changes.

module-member-permission-protection-name = Member Permission Protection
module-member-permission-protection-description = Protect members from being granted dangerous permissions via roles.

module-bot-adding-protection-name = Bot Adding Protection
module-bot-adding-protection-description = Automatically kicks newly added bots and punishes the user who added them.

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
log-field-action-status = Action Status
log-field-role-id = Role ID
log-field-role = Role
log-field-channel = Channel
log-field-user = User

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
