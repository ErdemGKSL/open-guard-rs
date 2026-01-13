# Discord Modals and Labels Documentation

## Overview

Modals are popup forms that allow you to collect structured input from users in Discord. Labels are layout components that wrap modal input components with descriptive text, providing context for what the user should enter.  

## What is a Modal?

A modal is an interactive popup form that can contain various input components. Modals are triggered in response to interactions (like clicking a button or using a slash command) and allow you to collect multiple pieces of information from users in a structured way.  

### Key Characteristics:

*   Modals are temporary overlays that require user interaction
*   They can contain multiple input fields
*   Users must either submit or dismiss the modal
*   Each modal has a `custom_id` and `title`

## Modal Structure

```json
{
  "type": 9,  // InteractionCallbackType.MODAL
  "data": {
    "custom_id": "my_modal",
    "title": "Modal Title",
    "components": [
      // Label components go here
    ]
  }
}
```

### Modal Fields:

*   **type**: Always `9` for modals
*   **custom\_id**: Developer-defined identifier (1-100 characters)
*   **title**: The modal's title shown at the top
*   **components**: Array of Label components

## What is a Label?

A Label is a top-level layout component that wraps modal input components with descriptive text. It provides a label and optional description for input fields, making forms more user-friendly.  

### Label Structure

```json
{
  "type": 18,  // ComponentType.LABEL
  "id": 1,     // Optional identifier
  "label": "Field Label",
  "description": "Optional description text",
  "component": {
    // Input component goes here
  }
}
```

### Label Fields:

*   **type**: Always `18` for labels
*   **id**: Optional identifier for the component
*   **label**: The label text (max 45 characters)
*   **description**: Optional description text (max 100 characters)
*   **component**: The input component wrapped by this label

> **Note**: The `description` may display above or below the `component` depending on the platform.  

## Supported Components in Labels

Labels can wrap the following interactive components:  

1.  **Text Input** - For text entry
2.  **String Select** - Dropdown with custom options
3.  **User Select** - Select Discord users
4.  **Role Select** - Select server roles
5.  **Mentionable Select** - Select users and/or roles
6.  **Channel Select** - Select server channels
7.  **File Upload** - Upload files

You can also use **Text Display** components directly in modals for informational text.  

## Complete Modal Example

Here's a full example of a modal with multiple input types:  

```json
{
  "type": 9,
  "data": {
    "custom_id": "feedback_modal",
    "title": "Feedback Form",
    "components": [
      {
        "type": 10,  // Text Display (informational text)
        "content": "Please provide your feedback below"
      },
      {
        "type": 18,  // Label
        "label": "Your Feedback",
        "description": "Tell us what you think",
        "component": {
          "type": 4,  // Text Input
          "custom_id": "feedback_text",
          "style": 2,  // Paragraph style
          "placeholder": "Write your feedback here...",
          "required": true
        }
      },
      {
        "type": 18,  // Label
        "label": "Rating",
        "component": {
          "type": 3,  // String Select
          "custom_id": "rating",
          "placeholder": "Select a rating",
          "options": [
            {
              "label": "Excellent",
              "value": "5"
            },
            {
              "label": "Good",
              "value": "4"
            },
            {
              "label": "Average",
              "value": "3"
            }
          ],
          "required": true
        }
      }
    ]
  }
}
```

## Receiving Modal Submissions

When a user submits a modal, you'll receive an interaction with `type: 5` (MODAL\_SUBMIT). The data structure looks like this:  

```json
{
  "type": 5,  // InteractionType.MODAL_SUBMIT
  "data": {
    "custom_id": "feedback_modal",
    "components": [
      {
        "type": 18,  // Label
        "id": 1,
        "component": {
          "type": 4,  // Text Input
          "id": 2,
          "custom_id": "feedback_text",
          "value": "Great app!"  // User's input
        }
      },
      {
        "type": 18,
        "id": 3,
        "component": {
          "type": 3,  // String Select
          "id": 4,
          "custom_id": "rating",
          "values": ["5"]  // Selected value
        }
      }
    ]
  }
}
```

## Common Input Components in Labels

### Text Input

```json
{
  "type": 18,
  "label": "Email Address",
  "description": "We'll never share your email",
  "component": {
    "type": 4,
    "custom_id": "user_email",
    "style": 1,  // Short (single line)
    "placeholder": "user@example.com",
    "required": true,
    "min_length": 5,
    "max_length": 100
  }
}
```

### String Select

```json
{
  "type": 18,
  "label": "Choose your favorite",
  "component": {
    "type": 3,
    "custom_id": "favorite_choice",
    "placeholder": "Select an option",
    "options": [
      {"label": "Option 1", "value": "opt1"},
      {"label": "Option 2", "value": "opt2"}
    ],
    "required": true
  }
}
```

### User Select

```json
{
  "type": 18,
  "label": "Nominate a team member",
  "component": {
    "type": 5,
    "custom_id": "nominated_user",
    "max_values": 3,
    "required": true
  }
}
```

### File Upload

```json
{
  "type": 18,
  "label": "Upload Screenshot",
  "description": "Please upload a screenshot of the issue",
  "component": {
    "type": 19,
    "custom_id": "screenshot_upload",
    "min_values": 1,
    "max_values": 5,
    "required": true
  }
}
```

## Best Practices

1.  **Keep labels concise**: Maximum 45 characters for better display
2.  **Use descriptions wisely**: Provide helpful context without overwhelming users
3.  **Set appropriate required flags**: Only require fields that are truly necessary
4.  **Use placeholder text**: Guide users on expected input format
5.  **Validate min/max values**: Set appropriate limits for text length and selection counts
6.  **Clear custom\_ids**: Use descriptive identifiers to easily process submissions

## Limitations

*   Modals can only be shown in response to interactions
*   Each modal can contain multiple Label components
*   You cannot currently disable components in modals (disabled fields will cause errors)
*   The `required` field defaults to `true` for most components

## Legacy vs. Modern Approach

**Deprecated**: Action Rows with Text Inputs in modals  

```json
// Don't use this approach anymore
{
  "type": 1,  // Action Row
  "components": [{
    "type": 4,
    "custom_id": "input",
    "label": "Label"
  }]
}
```

**Recommended**: Labels wrapping components  

```json
// Use this approach
{
  "type": 18,  // Label
  "label": "Label",
  "component": {
    "type": 4,
    "custom_id": "input"
  }
}
```

The Label component provides better organization and allows for descriptions, making forms more user-friendly and accessible.

### Serenity implementation
```rs
use std::borrow::Cow;

use serde::Serialize;

use crate::model::prelude::*;

/// A builder for creating a label that can hold an [`InputText`] or [`SelectMenu`].
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#label).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateLabel<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    label: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    component: CreateLabelComponent<'a>,
}

impl<'a> CreateLabel<'a> {
    /// Create a select menu with a specific label.
    pub fn select_menu(label: impl Into<Cow<'a, str>>, select_menu: CreateSelectMenu<'a>) -> Self {
        Self {
            kind: ComponentType::Label,
            label: label.into(),
            description: None,
            component: CreateLabelComponent::SelectMenu(select_menu),
        }
    }

    /// Create a text input with a specific label.
    pub fn input_text(label: impl Into<Cow<'a, str>>, input_text: CreateInputText<'a>) -> Self {
        Self {
            kind: ComponentType::Label,
            label: label.into(),
            description: None,
            component: CreateLabelComponent::InputText(input_text),
        }
    }

    /// Create a file upload with a specific label.
    pub fn file_upload(label: impl Into<Cow<'a, str>>, file_upload: CreateFileUpload<'a>) -> Self {
        Self {
            kind: ComponentType::Label,
            label: label.into(),
            description: None,
            component: CreateLabelComponent::FileUpload(file_upload),
        }
    }

    /// Sets the description of this component, which will display underneath the label text.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// An enum of all valid label components.
#[derive(Clone, Debug, Serialize)]
#[must_use]
#[serde(untagged)]
enum CreateLabelComponent<'a> {
    SelectMenu(CreateSelectMenu<'a>),
    InputText(CreateInputText<'a>),
    FileUpload(CreateFileUpload<'a>),
}

/// A builder for creating a file upload in a modal.
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#file-upload).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateFileUpload<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    custom_id: Cow<'a, str>,
    min_values: u8,
    max_values: u8,
    required: bool,
}

impl<'a> CreateFileUpload<'a> {
    /// Creates a builder with the given custom id.
    pub fn new(custom_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: ComponentType::FileUpload,
            custom_id: custom_id.into(),
            min_values: 1,
            max_values: 1,
            required: true,
        }
    }

    /// The minimum number of files that must be uploaded. Must be a number from 0 through 10, and
    /// defaults to 1.
    pub fn min_values(mut self, min_values: u8) -> Self {
        self.min_values = min_values;
        self
    }

    /// The maximum number of files that can be uploaded. Defaults to 1, but can be at most 10.
    pub fn max_values(mut self, max_values: u8) -> Self {
        self.max_values = max_values;
        self
    }

    // Whether the file upload is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

enum_number! {
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum Spacing {
        Small = 1,
        Large = 2,
        _ => Unknown(u8),
    }
}

struct CreateSelectMenuDefault(Mention);

impl Serialize for CreateSelectMenuDefault {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let (id, kind) = match self.0 {
            Mention::Channel(c) => (c.get(), "channel"),
            Mention::Role(r) => (r.get(), "role"),
            Mention::User(u) => (u.get(), "user"),
        };

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("id", &id)?;
        map.serialize_entry("type", kind)?;
        map.end()
    }
}

/// [Discord docs](https://discord.com/developers/docs/components/reference#component-object-component-types).
#[derive(Clone, Debug)]
pub enum CreateSelectMenuKind<'a> {
    String {
        options: Cow<'a, [CreateSelectMenuOption<'a>]>,
    },
    User {
        default_users: Option<Cow<'a, [UserId]>>,
    },
    Role {
        default_roles: Option<Cow<'a, [RoleId]>>,
    },
    Mentionable {
        default_users: Option<Cow<'a, [UserId]>>,
        default_roles: Option<Cow<'a, [RoleId]>>,
    },
    Channel {
        channel_types: Option<Cow<'a, [ChannelType]>>,
        default_channels: Option<Cow<'a, [GenericChannelId]>>,
    },
}

impl Serialize for CreateSelectMenuKind<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Json<'a> {
            #[serde(rename = "type")]
            kind: u8,
            #[serde(skip_serializing_if = "Option::is_none")]
            options: Option<&'a [CreateSelectMenuOption<'a>]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            channel_types: Option<&'a [ChannelType]>,
            #[serde(skip_serializing_if = "<[_]>::is_empty")]
            default_values: &'a [CreateSelectMenuDefault],
        }

        #[expect(clippy::ref_option)]
        fn map<'a>(
            values: &'a Option<Cow<'a, [impl Into<Mention> + Copy]>>,
        ) -> impl Iterator<Item = CreateSelectMenuDefault> + 'a {
            // Calling `.iter().flatten()` on the `Option` treats `None` like an empty vec
            values
                .as_ref()
                .map(|s| s.iter())
                .into_iter()
                .flatten()
                .map(|&i| CreateSelectMenuDefault(i.into()))
        }

        #[rustfmt::skip]
        let default_values = match self {
            Self::String { .. } => vec![],
            Self::User { default_users: default_values } => map(default_values).collect(),
            Self::Role { default_roles: default_values } => map(default_values).collect(),
            Self::Mentionable { default_users, default_roles } => {
                let users = map(default_users);
                let roles = map(default_roles);
                users.chain(roles).collect()
            },
            Self::Channel { channel_types: _, default_channels: default_values } => map(default_values).collect(),
        };

        #[rustfmt::skip]
        let json = Json {
            kind: match self {
                Self::String { .. } => 3,
                Self::User { .. } => 5,
                Self::Role { .. } => 6,
                Self::Mentionable { .. } => 7,
                Self::Channel { .. } => 8,
            },
            options: match self {
                Self::String { options } => Some(options),
                _ => None,
            },
            channel_types: match self {
                Self::Channel { channel_types, default_channels: _ } => channel_types.as_deref(),
                _ => None,
            },
            default_values: &default_values,
        };

        json.serialize(serializer)
    }
}

/// A builder for creating a select menu component in a message
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#component-object-component-types).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenu<'a> {
    custom_id: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,

    #[serde(flatten)]
    kind: CreateSelectMenuKind<'a>,
}

impl<'a> CreateSelectMenu<'a> {
    /// Creates a builder with given custom id (a developer-defined identifier), and a list of
    /// options, leaving all other fields empty.
    pub fn new(custom_id: impl Into<Cow<'a, str>>, kind: CreateSelectMenuKind<'a>) -> Self {
        Self {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            required: None,
            disabled: None,
            kind,
        }
    }

    /// The placeholder of the select menu.
    pub fn placeholder(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<Cow<'a, str>>) -> Self {
        self.custom_id = id.into();
        self
    }

    /// Sets the minimum values for the user to select.
    pub fn min_values(mut self, min: u8) -> Self {
        self.min_values = Some(min);
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(mut self, max: u8) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Sets the required state for the select menu in modals. Ignored in messages.
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Sets the disabled state for the select menu.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}

/// A builder for creating an option of a select menu component in a message
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#string-select-select-option-structure)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSelectMenuOption<'a> {
    label: Cow<'a, str>,
    value: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
}

impl<'a> CreateSelectMenuOption<'a> {
    /// Creates a select menu option with the given label and value, leaving all other fields
    /// empty.
    pub fn new(label: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: None,
        }
    }

    /// Sets the label of this option, replacing the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the value of this option, replacing the current value as set in [`Self::new`].
    pub fn value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = value.into();
        self
    }

    /// Sets the description shown on this option.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets emoji of the option.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    /// Sets this option as selected by default.
    pub fn default_selection(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }
}

/// A builder for creating an input text component in a modal
///
/// [Discord docs](https://discord.com/developers/docs/components/reference#text-input).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateInputText<'a> {
    #[serde(rename = "type")]
    kind: ComponentType,
    custom_id: Cow<'a, str>,
    style: InputTextStyle,
    min_length: Option<u16>,
    max_length: Option<u16>,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<Cow<'a, str>>,
}

impl<'a> CreateInputText<'a> {
    /// Creates a text input with the given style, label, and custom id (a developer-defined
    /// identifier), leaving all other fields empty.
    pub fn new(style: InputTextStyle, custom_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            style,
            custom_id: custom_id.into(),

            placeholder: None,
            min_length: None,
            max_length: None,
            value: None,
            required: true,

            kind: ComponentType::InputText,
        }
    }

    /// Sets the style of this input text. Replaces the current value as set in [`Self::new`].
    pub fn style(mut self, kind: InputTextStyle) -> Self {
        self.style = kind;
        self
    }

    /// Sets the custom id of the input text, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<Cow<'a, str>>) -> Self {
        self.custom_id = id.into();
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(mut self, min: u16) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(mut self, max: u16) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Sets the value of this input text.
    pub fn value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets if the input text is required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}
```