import { useCallback, useState } from 'react';
import { getAppServerClient } from '@/lib/appServerClient';

export interface PlanStepDto {
  id: string;
  name: string;
  stepType: string;
  status: string;
}

export interface ExecutionPlanDto {
  id: string;
  steps: PlanStepDto[];
}

export interface WorkflowStartResponse {
  workflowId: string;
  status: string;
  plan: ExecutionPlanDto | null;
}

export interface WorkflowStatusResponse {
  workflowId: string;
  status: string;
  progress: number;
  completedSteps: string[];
  failedSteps: string[];
  errors: string[];
}

export interface WorkflowResumeResponse {
  workflowId: string;
  status: string;
}

export interface UseWorkflowOptions {
  connectionReady: boolean;
}

export function useWorkflow({ connectionReady }: UseWorkflowOptions) {
  const [activeWorkflowId, setActiveWorkflowId] = useState<string | null>(null);
  const [workflowStatus, setWorkflowStatus] = useState<string | null>(null);
  const [plan, setPlan] = useState<ExecutionPlanDto | null>(null);
  const [progress, setProgress] = useState(0);
  const [completedSteps, setCompletedSteps] = useState<string[]>([]);
  const [failedSteps, setFailedSteps] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const startWorkflow = useCallback(
    async (
      goal: string,
      opts?: {
        templateId?: string;
        model?: string;
        maxRetries?: number;
      },
    ) => {
      if (!connectionReady) return null;
      setLoading(true);
      setError(null);
      try {
        const client = getAppServerClient();
        const res = await client.request<WorkflowStartResponse>(
          'workflow/start',
          {
            goal,
            templateId: opts?.templateId ?? null,
            model: opts?.model ?? null,
            maxRetries: opts?.maxRetries ?? null,
          },
        );
        setActiveWorkflowId(res.workflowId);
        setWorkflowStatus(res.status);
        setPlan(res.plan);
        return res;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        return null;
      } finally {
        setLoading(false);
      }
    },
    [connectionReady],
  );

  const resumeWorkflow = useCallback(async () => {
    if (!connectionReady || !activeWorkflowId) return null;
    setLoading(true);
    setError(null);
    try {
      const client = getAppServerClient();
      const res = await client.request<WorkflowResumeResponse>(
        'workflow/resume',
        { workflowId: activeWorkflowId },
      );
      setWorkflowStatus(res.status);
      return res;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      return null;
    } finally {
      setLoading(false);
    }
  }, [connectionReady, activeWorkflowId]);

  const fetchStatus = useCallback(async () => {
    if (!connectionReady || !activeWorkflowId) return null;
    try {
      const client = getAppServerClient();
      const res = await client.request<WorkflowStatusResponse>(
        'workflow/status',
        { workflowId: activeWorkflowId },
      );
      setWorkflowStatus(res.status);
      setProgress(res.progress);
      setCompletedSteps(res.completedSteps);
      setFailedSteps(res.failedSteps);
      if (res.errors.length > 0) {
        setError(res.errors.join('; '));
      }
      return res;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      return null;
    }
  }, [connectionReady, activeWorkflowId]);

  const reset = useCallback(() => {
    setActiveWorkflowId(null);
    setWorkflowStatus(null);
    setPlan(null);
    setProgress(0);
    setCompletedSteps([]);
    setFailedSteps([]);
    setError(null);
  }, []);

  return {
    activeWorkflowId,
    workflowStatus,
    plan,
    progress,
    completedSteps,
    failedSteps,
    loading,
    error,
    startWorkflow,
    resumeWorkflow,
    fetchStatus,
    reset,
  };
}
