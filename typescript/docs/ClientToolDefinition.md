
# ClientToolDefinition

User-defined tool definition sent via the API request.  Uses JSON Schema format compatible with OpenAI/Anthropic function calling. Tool definitions are passed to the LLM alongside server built-in tools so the model can decide when to invoke them.  # Example  ```json {   \"name\": \"query_database\",   \"description\": \"Run a read-only SQL query\",   \"parameters\": {     \"type\": \"object\",     \"properties\": {       \"sql\": { \"type\": \"string\" }     },     \"required\": [\"sql\"]   },   \"fire_and_forget\": false } ```

## Properties

Name | Type
------------ | -------------
`description` | string
`fireAndForget` | boolean
`name` | string
`parameters` | any

## Example

```typescript
import type { ClientToolDefinition } from '@tachyon/sdk'

// TODO: Update the object below with actual values
const example = {
  "description": null,
  "fireAndForget": null,
  "name": null,
  "parameters": null,
} satisfies ClientToolDefinition

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ClientToolDefinition
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


