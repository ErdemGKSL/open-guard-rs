---
name: components-v2-content
description: Guide for content display components (Text, Thumbnail, Media, File) in Discord Components v2
metadata:
  version: "2.0"
---

# Content Components (Components v2)

## What this skill does
Reference for displaying static content using Text Display, Thumbnail, Media Gallery, and File components.

## When to use
- Display formatted text with Markdown
- Show images and thumbnails
- Display media galleries
- Embed file attachments
- Replace legacy embed fields with modern components

---

# Text Display (Type 10)

## What is Text Display?
The primary component for displaying text content. Replaces message `content` field and embed descriptions.

**Supports Markdown**: Bold, italic, code blocks, headings, links, etc.

## Creating Text Display

### Basic Text
```rust
use poise::serenity_prelude as serenity;

serenity::CreateTextDisplay::new("Hello World!")
```

### Markdown Text
```rust
serenity::CreateTextDisplay::new("**Bold** *italic* `code`")
```

### Headings
```rust
// H1
serenity::CreateTextDisplay::new("# Main Title")

// H2
serenity::CreateTextDisplay::new("## Subtitle")

// H3
serenity::CreateTextDisplay::new("### Section")
```

### Multi-line Text
```rust
serenity::CreateTextDisplay::new(format!(
    "**Status**: Enabled\n\
    **Users**: 42\n\
    **Last Updated**: <t:{}:R>",
    timestamp
))
```

### Dynamic Content
```rust
serenity::CreateTextDisplay::new(format!(
    "Welcome <@{}>!\nYour role: {}",
    user_id,
    role_name
))
```

## Usage in Container

```rust
let mut components = vec![];

// Title
components.push(serenity::CreateContainerComponent::TextDisplay(
    serenity::CreateTextDisplay::new("## Configuration")
));

// Description
components.push(serenity::CreateContainerComponent::TextDisplay(
    serenity::CreateTextDisplay::new("Configure your server settings below.")
));

// Field-style content
components.push(serenity::CreateContainerComponent::TextDisplay(
    serenity::CreateTextDisplay::new(format!(
        "**Guild ID**: `{}`\n\
        **Owner**: <@{}>\n\
        **Members**: {}",
        guild_id, owner_id, member_count
    ))
));
```

## Usage in Section

```rust
// From src/services/config/builders.rs
serenity::CreateSection::new(
    vec![
        serenity::CreateSectionComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "**{}**\n{}",
                module_name,
                description
            ))
        ),
    ],
    accessory
)
```

## Common Patterns

### Status Display
```rust
// From src/services/status.rs
serenity::CreateTextDisplay::new(format!(
    "**Status**: {}\n\
    **Uptime**: {}\n\
    **Guilds**: {}",
    if healthy { "✅ Online" } else { "❌ Offline" },
    uptime_str,
    guild_count
))
```

### Field List
```rust
serenity::CreateTextDisplay::new(format!(
    "**User**: <@{}>\n\
    **Action**: {}\n\
    **Reason**: {}\n\
    **Timestamp**: <t:{}:F>",
    user_id, action, reason, timestamp
))
```

### Code Block
```rust
serenity::CreateTextDisplay::new(format!(
    "```json\n{}\n```",
    serde_json::to_string_pretty(&data)?
))
```

---

# Thumbnail (Type 11)

## What is Thumbnail?
Displays a small image, typically used as a Section accessory.

**Status in Codebase**: ❌ Not yet used (but supported by Serenity)

## Creating Thumbnail

### Basic Thumbnail
```rust
serenity::CreateThumbnail::new("https://example.com/image.png")
```

### As Section Accessory
```rust
serenity::CreateSection::new(
    vec![
        serenity::CreateSectionComponent::TextDisplay(
            serenity::CreateTextDisplay::new("**User Profile**")
        ),
    ],
    serenity::CreateSectionAccessory::Thumbnail(
        serenity::CreateThumbnail::new(user_avatar_url)
    ),
)
```

## Potential Use Cases

### User Profile
```rust
serenity::CreateSection::new(
    vec![
        serenity::CreateSectionComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "**{}**\nJoined: <t:{}:R>",
                user.name, joined_timestamp
            ))
        ),
    ],
    serenity::CreateSectionAccessory::Thumbnail(
        serenity::CreateThumbnail::new(user.avatar_url())
    ),
)
```

### Server Info
```rust
serenity::CreateSection::new(
    vec![
        serenity::CreateSectionComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!(
                "**{}**\nMembers: {}",
                guild.name, guild.member_count
            ))
        ),
    ],
    serenity::CreateSectionAccessory::Thumbnail(
        serenity::CreateThumbnail::new(guild.icon_url())
    ),
)
```

---

# Media Gallery (Type 12)

## What is Media Gallery?
Displays multiple images or videos in a gallery layout.

**Status in Codebase**: ❌ Not yet used (but supported by Serenity)

## Creating Media Gallery

### Basic Gallery
```rust
serenity::CreateMediaGallery::new(vec![
    serenity::CreateMediaGalleryItem::new("https://example.com/image1.png"),
    serenity::CreateMediaGalleryItem::new("https://example.com/image2.png"),
])
```

### Multiple Images
```rust
let items: Vec<_> = image_urls
    .iter()
    .map(|url| serenity::CreateMediaGalleryItem::new(url))
    .collect();

serenity::CreateMediaGallery::new(items)
```

## Usage in Container
```rust
components.push(serenity::CreateContainerComponent::MediaGallery(
    serenity::CreateMediaGallery::new(vec![
        serenity::CreateMediaGalleryItem::new("https://cdn.discordapp.com/attachments/.../image1.png"),
        serenity::CreateMediaGalleryItem::new("https://cdn.discordapp.com/attachments/.../image2.png"),
    ])
));
```

## Potential Use Cases

### Violation Evidence
```rust
// Show screenshots of rule violations
let screenshots = vec![
    "https://cdn.example.com/screenshot1.png",
    "https://cdn.example.com/screenshot2.png",
];

let gallery_items: Vec<_> = screenshots
    .iter()
    .map(|url| serenity::CreateMediaGalleryItem::new(*url))
    .collect();

components.push(serenity::CreateContainerComponent::MediaGallery(
    serenity::CreateMediaGallery::new(gallery_items)
));
```

### Audit Log Attachments
```rust
// Display before/after images in moderation logs
serenity::CreateMediaGallery::new(vec![
    serenity::CreateMediaGalleryItem::new(before_image_url),
    serenity::CreateMediaGalleryItem::new(after_image_url),
])
```

---

# File (Type 13)

## What is File?
Embeds a file attachment directly into the message body.

**Status in Codebase**: ❌ Not yet used (but supported by Serenity)

## Creating File Component

### Basic File
```rust
serenity::CreateFile::new(
    "https://example.com/document.pdf",
    "document.pdf"
)
```

### With Custom Filename
```rust
serenity::CreateFile::new(
    attachment_url,
    format!("export_{}.json", timestamp)
)
```

## Usage in Container
```rust
components.push(serenity::CreateContainerComponent::File(
    serenity::CreateFile::new(
        "https://cdn.discordapp.com/attachments/.../config.json",
        "server_config.json"
    )
));
```

## Potential Use Cases

### Configuration Export
```rust
// Export server configuration as JSON
let config_json = serde_json::to_string_pretty(&guild_config)?;
// Upload to Discord CDN first, then:
components.push(serenity::CreateContainerComponent::File(
    serenity::CreateFile::new(uploaded_url, "config.json")
));
```

### Log Export
```rust
// Attach audit log file
components.push(serenity::CreateContainerComponent::File(
    serenity::CreateFile::new(
        log_file_url,
        format!("audit_log_{}.txt", guild_id)
    )
));
```

### Backup Files
```rust
// Provide downloadable backup
serenity::CreateFile::new(
    backup_url,
    format!("backup_{}_{}.zip", guild_id, timestamp)
)
```

---

# Complete Content Example

```rust
pub fn build_user_profile(
    user: &serenity::User,
    joined_at: i64,
    roles: &[String],
) -> Vec<serenity::CreateComponent> {
    let mut inner_components = vec![];
    
    // Header with user avatar (if Thumbnail was used)
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!("## {}", user.name))
    ));
    
    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false)
    ));
    
    // User info
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "**ID**: `{}`\n\
            **Joined**: <t:{}:R>\n\
            **Mention**: <@{}>",
            user.id, joined_at, user.id
        ))
    ));
    
    // Separator
    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true)
    ));
    
    // Roles section
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new("**Roles**")
    ));
    
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(roles.join(", "))
    ));
    
    vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components)
            .accent_color(0x5865F2)
    )]
}
```

---

# Markdown Reference

Text Display supports Discord Markdown:

```
**bold**
*italic*
__underline__
~~strikethrough~~
`code`
```code block```
# Heading 1
## Heading 2
### Heading 3
[link](https://example.com)
<@user_id> - User mention
<@&role_id> - Role mention
<#channel_id> - Channel mention
<t:timestamp:F> - Timestamp (full date)
<t:timestamp:R> - Timestamp (relative)
```

---

# Best Practices

1. **Use Text Display everywhere** - Replaces content/embed description
2. **Markdown for formatting** - Bold headers, italic notes
3. **Mention syntax** - Use `<@user_id>` not `@username`
4. **Timestamps** - Use `<t:unix:format>` for dates
5. **Code blocks** - Triple backticks for formatted code/JSON
6. **Multi-line for fields** - Use `\n` for field-style layouts
7. **Thumbnail for avatars** - Small images as section accessories
8. **Media Gallery for multiple** - Use gallery for 2+ images
9. **File for downloads** - Embed downloadable attachments

---

# Codebase Examples

**Text Display:**
- `src/services/config/mod.rs:58-59` - Simple label
- `src/services/status.rs:55-56` - Heading with Markdown
- `src/services/logger.rs:92-93` - Multi-line content
- `src/services/config/builders.rs:63-64` - Formatted field

**Separators:**
- `src/services/config/mod.rs:73-74` - Non-spacing
- `src/services/logger.rs:96-97` - Spacing separator

**Container Usage:**
- `src/services/logger.rs:120-122` - Logger message
- `src/services/status.rs:166-168` - Status display
- `src/services/help.rs:78-80` - Help menu

---

# Migration from Legacy Embeds

### Old (Legacy Embed)
```rust
ctx.send(poise::CreateReply::default()
    .embed(serenity::CreateEmbed::new()
        .title("Title")
        .description("Description")
        .field("Field 1", "Value 1", false)
        .color(0x5865F2)
    )
).await?;
```

### New (Components v2)
```rust
let mut components = vec![];

components.push(serenity::CreateContainerComponent::TextDisplay(
    serenity::CreateTextDisplay::new("## Title")
));

components.push(serenity::CreateContainerComponent::TextDisplay(
    serenity::CreateTextDisplay::new("Description")
));

components.push(serenity::CreateContainerComponent::Separator(
    serenity::CreateSeparator::new(false)
));

components.push(serenity::CreateContainerComponent::TextDisplay(
    serenity::CreateTextDisplay::new("**Field 1**\nValue 1")
));

ctx.send(poise::CreateReply::default()
    .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
    .components(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(components)
            .accent_color(0x5865F2)
    )])
).await?;
```

---

# See Also
- `components-v2-layout` - Container, Section, Separator
- `components-v2-interactive` - Buttons, Selects, Action Rows
- `components-v2-modals` - Modals with Labels
