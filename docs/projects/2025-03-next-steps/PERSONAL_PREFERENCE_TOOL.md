# Personal Preference Tool Specification

## Overview

The Personal Preference Tool is a feature for the progmo-mcp-server that captures, stores, and provides developer preferences and style decisions to coding assistants. This tool aims to improve the consistency and personalization of code generation by maintaining a record of a developer's preferences and making them available as context when relevant.

## Purpose

- Capture developer preferences and style decisions during coding sessions
- Store these preferences in a structured, queryable format
- Provide relevant preferences to coding assistants when generating code
- Reduce the need for repetitive style instructions
- Improve consistency across projects and coding sessions

## Core Features

### 1. Preference Capture

- **Explicit Capture**: Allow developers to explicitly define preferences through commands or API
- **Implicit Capture**: Analyze developer feedback on generated code to infer preferences
- **Preference Categories**:
  - Code style (indentation, bracket placement, naming conventions)
  - Architecture preferences (design patterns, project structure)
  - Technology choices (libraries, frameworks, tools)
  - Documentation style (comment format, documentation level)
  - Testing approach (test frameworks, coverage expectations)

### 2. Preference Storage

- **Storage Format**: JSON-based preference store with hierarchical organization
- **Versioning**: Track changes to preferences over time
- **Scoping**:
  - Global preferences (apply to all projects)
  - Project-specific preferences (override globals for specific projects)
  - Language-specific preferences (apply to specific programming languages)
  - Context-specific preferences (apply in specific coding contexts)

### 3. Preference Retrieval

- **Context-Aware Retrieval**: Provide only preferences relevant to the current task
- **Priority System**: Resolve conflicts between overlapping preferences
- **Query Interface**: Allow coding assistants to query for specific preference types
- **Bulk Retrieval**: Provide all relevant preferences for a given context

### 4. Integration with MCP

- **MCP Tool**: Implement as an MCP tool with standard request/response format
- **Context Injection**: Automatically inject relevant preferences into coding assistant context
- **Feedback Loop**: Update preferences based on developer feedback

## API Design

### MCP Tool: `get_preferences`

**Purpose**: Retrieve relevant preferences for a given context

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "project": {
      "type": "string",
      "description": "Project identifier"
    },
    "language": {
      "type": "string",
      "description": "Programming language"
    },
    "context": {
      "type": "string",
      "description": "Current coding context (e.g., 'web-frontend', 'api-design')"
    },
    "categories": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "description": "Specific preference categories to retrieve"
    }
  },
  "required": []
}
```

**Output Format**:
```json
{
  "preferences": {
    "style": {
      "indentation": "2 spaces",
      "lineLength": 80,
      "quoteStyle": "single",
      "bracketStyle": "same-line"
    },
    "naming": {
      "variables": "camelCase",
      "constants": "UPPER_SNAKE_CASE",
      "functions": "camelCase",
      "classes": "PascalCase"
    },
    "architecture": {
      "preferredPatterns": ["repository", "dependency-injection"],
      "avoidPatterns": ["singleton"]
    },
    "documentation": {
      "commentStyle": "JSDoc",
      "requireParamDocs": true
    }
  },
  "source": {
    "global": ["style.indentation", "style.lineLength"],
    "project": ["naming.variables", "architecture.preferredPatterns"],
    "language": ["documentation.commentStyle"]
  }
}
```

### MCP Tool: `set_preference`

**Purpose**: Set or update a developer preference

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Dot-notation path to the preference (e.g., 'style.indentation')"
    },
    "value": {
      "type": "any",
      "description": "The preference value"
    },
    "scope": {
      "type": "object",
      "properties": {
        "project": {
          "type": "string",
          "description": "Project identifier for project-scoped preferences"
        },
        "language": {
          "type": "string",
          "description": "Programming language for language-scoped preferences"
        },
        "context": {
          "type": "string",
          "description": "Context for context-scoped preferences"
        }
      }
    }
  },
  "required": ["path", "value"]
}
```

**Output Format**:
```json
{
  "success": true,
  "path": "style.indentation",
  "value": "2 spaces",
  "scope": {
    "type": "global"
  }
}
```

### MCP Tool: `infer_preferences`

**Purpose**: Analyze code or feedback to infer preferences

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "code": {
      "type": "string",
      "description": "Code sample to analyze for style preferences"
    },
    "feedback": {
      "type": "string",
      "description": "Developer feedback to analyze for preference hints"
    },
    "language": {
      "type": "string",
      "description": "Programming language of the code"
    },
    "project": {
      "type": "string",
      "description": "Project identifier"
    }
  },
  "required": ["language"]
}
```

**Output Format**:
```json
{
  "inferred": {
    "style.indentation": "4 spaces",
    "style.quoteStyle": "double",
    "naming.variables": "camelCase"
  },
  "confidence": {
    "style.indentation": 0.95,
    "style.quoteStyle": 0.8,
    "naming.variables": 0.9
  },
  "applied": ["style.indentation", "naming.variables"],
  "skipped": {
    "style.quoteStyle": "confidence below threshold"
  }
}
```

## Implementation Plan

### Phase 1: Core Infrastructure

1. Design and implement preference storage system
2. Create basic MCP tools for getting and setting preferences
3. Implement preference scoping and resolution logic
4. Add integration with coding assistant context

### Phase 2: Preference Inference

1. Implement code analysis for style inference
2. Add feedback analysis for preference extraction
3. Create confidence scoring system for inferred preferences
4. Develop automatic preference application rules

### Phase 3: Advanced Features

1. Add preference versioning and history
2. Implement preference conflict resolution
3. Create preference templates for common styles (e.g., Google style, Airbnb style)
4. Add preference sharing and team preference support

## Data Model

### Preference Store

```typescript
interface PreferenceStore {
  global: {
    style: StylePreferences;
    naming: NamingPreferences;
    architecture: ArchitecturePreferences;
    documentation: DocumentationPreferences;
    testing: TestingPreferences;
  };
  projects: {
    [projectId: string]: ProjectPreferences;
  };
  languages: {
    [language: string]: LanguagePreferences;
  };
  contexts: {
    [context: string]: ContextPreferences;
  };
}

interface StylePreferences {
  indentation: string;
  lineLength: number;
  quoteStyle: "single" | "double";
  bracketStyle: "same-line" | "new-line";
  trailingComma: boolean;
  semicolons: boolean;
}

interface NamingPreferences {
  variables: string;
  constants: string;
  functions: string;
  classes: string;
  interfaces: string;
  files: string;
  directories: string;
}

interface ArchitecturePreferences {
  preferredPatterns: string[];
  avoidPatterns: string[];
  folderStructure: string;
  moduleOrganization: string;
}

interface DocumentationPreferences {
  commentStyle: string;
  requireParamDocs: boolean;
  requireReturnDocs: boolean;
  requireClassDocs: boolean;
  docFormat: string;
}

interface TestingPreferences {
  framework: string;
  coverageThreshold: number;
  testNamingPattern: string;
  testLocation: string;
}

interface ProjectPreferences extends Partial<PreferenceStore["global"]> {
  projectId: string;
}

interface LanguagePreferences extends Partial<PreferenceStore["global"]> {
  language: string;
}

interface ContextPreferences extends Partial<PreferenceStore["global"]> {
  context: string;
}
```

## Success Criteria

The Personal Preference Tool will be considered successful when:

1. Developers can define and retrieve their coding preferences
2. Coding assistants consistently apply the defined preferences
3. The system can infer preferences from code and feedback with high accuracy
4. Preferences are properly scoped and prioritized
5. The tool integrates seamlessly with the MCP protocol
6. Developers report improved consistency and reduced need for repetitive style instructions

## Future Enhancements

- **Team Preferences**: Support for team-wide preference sets
- **IDE Integration**: Direct integration with popular IDEs
- **Preference Analytics**: Insights into preference patterns and trends
- **Linter Integration**: Two-way sync with linter configurations
- **Preference Recommendations**: Suggest preferences based on project type or team norms
- **Natural Language Interface**: Allow setting preferences through natural language
