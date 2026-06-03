import type { McpElicitationPrimitiveSchema } from '@/generated/app-server/v2/McpElicitationPrimitiveSchema';
import type { McpElicitationSchema } from '@/generated/app-server/v2/McpElicitationSchema';
import type { McpServerElicitationRequestParams } from '@/generated/app-server/v2/McpServerElicitationRequestParams';
import type { JsonValue } from '@/generated/app-server/serde_json/JsonValue';
import type { ElicitationField, McpElicitation } from '@/types';

function primitiveToField(
  key: string,
  schema: McpElicitationPrimitiveSchema,
): ElicitationField | null {
  if ('enum' in schema && schema.type === 'string') {
    return {
      id: key,
      label: schema.title ?? key,
      type: 'select',
      description: schema.description,
      options: [...schema.enum],
    };
  }
  if (schema.type === 'string') {
    return {
      id: key,
      label: schema.title ?? key,
      type: 'text',
      description: schema.description,
    };
  }
  if (schema.type === 'boolean') {
    return {
      id: key,
      label: schema.title ?? key,
      type: 'select',
      description: schema.description,
      options: ['true', 'false'],
      value: schema.default === true ? 'true' : schema.default === false ? 'false' : undefined,
    };
  }
  if (schema.type === 'number' || schema.type === 'integer') {
    return {
      id: key,
      label: schema.title ?? key,
      type: 'text',
      description: schema.description,
      value: schema.default !== undefined ? String(schema.default) : undefined,
    };
  }
  return null;
}

function schemaToFields(requestedSchema: McpElicitationSchema): ElicitationField[] {
  const fields: ElicitationField[] = [];
  for (const [key, prop] of Object.entries(requestedSchema.properties)) {
    if (!prop) {
      continue;
    }
    const field = primitiveToField(key, prop);
    if (field) {
      fields.push(field);
    }
  }
  return fields;
}

export function mcpElicitationFromRpc(
  rpcId: string | number,
  params: McpServerElicitationRequestParams,
): McpElicitation {
  if (params.mode === 'url') {
    return {
      rpcId,
      rpcMethod: 'mcpServer/elicitation/request',
      serverName: params.serverName,
      message: params.message,
      mode: 'url',
      url: params.url,
      elicitationId: params.elicitationId,
      fields: [],
    };
  }
  return {
    rpcId,
    rpcMethod: 'mcpServer/elicitation/request',
    serverName: params.serverName,
    message: params.message,
    mode: 'form',
    fields: schemaToFields(params.requestedSchema),
  };
}

export function buildElicitationContent(
  fields: ElicitationField[],
  values: Record<string, string>,
): JsonValue {
  const content: Record<string, JsonValue> = {};
  for (const field of fields) {
    const raw = values[field.id] ?? field.value ?? '';
    if (field.type === 'select' && field.options?.includes('true') && field.options.includes('false')) {
      content[field.id] = raw === 'true';
      continue;
    }
    const asNumber = Number(raw);
    if (raw !== '' && !Number.isNaN(asNumber) && /^-?\d+(\.\d+)?$/.test(raw.trim())) {
      content[field.id] = asNumber;
      continue;
    }
    content[field.id] = raw;
  }
  return content;
}
