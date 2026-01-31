import { useEffect, useMemo, useRef, useState } from 'react';
import { Virtuoso, VirtuosoHandle } from 'react-virtuoso';

import DisplayConversationEntry from '../NormalizedConversation/DisplayConversationEntry';
import { useEntries } from '@/contexts/EntriesContext';
import {
  AddEntryType,
  PatchTypeWithKey,
  useConversationHistory,
} from '@/hooks/useConversationHistory';
import { Loader2 } from 'lucide-react';
import { TaskWithAttemptStatus } from 'shared/types';
import type { WorkspaceWithSession } from '@/types/attempt';
import { ApprovalFormProvider } from '@/contexts/ApprovalFormContext';

interface VirtualizedListProps {
  attempt: WorkspaceWithSession;
  task?: TaskWithAttemptStatus;
}

interface MessageListContext {
  attempt: WorkspaceWithSession;
  task?: TaskWithAttemptStatus;
}

const VirtualizedList = ({ attempt, task }: VirtualizedListProps) => {
  const [entries, setEntriesState] = useState<PatchTypeWithKey[]>([]);
  const [loading, setLoading] = useState(true);
  const { setEntries, reset } = useEntries();
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const didInitScroll = useRef(false);
  const [atBottom, setAtBottom] = useState(true);

  useEffect(() => {
    setLoading(true);
    setEntriesState([]);
    reset();
    didInitScroll.current = false;
  }, [attempt.id, reset]);

  // Initial jump to bottom once data appears
  useEffect(() => {
    if (!didInitScroll.current && entries.length > 0 && !loading) {
      didInitScroll.current = true;
      requestAnimationFrame(() => {
        virtuosoRef.current?.scrollToIndex({
          index: entries.length - 1,
          align: 'end',
        });
      });
    }
  }, [entries.length, loading]);

  const onEntriesUpdated = (
    newEntries: PatchTypeWithKey[],
    addType: AddEntryType,
    newLoading: boolean
  ) => {
    setEntriesState(newEntries);
    setEntries(newEntries);

    if (loading) {
      setLoading(newLoading);
    }

    // Auto-scroll to bottom when new entries are added during running
    if (addType === 'running' && !newLoading && atBottom && newEntries.length > 0) {
      requestAnimationFrame(() => {
        virtuosoRef.current?.scrollToIndex({
          index: newEntries.length - 1,
          align: 'end',
          behavior: 'smooth',
        });
      });
    }
  };

  useConversationHistory({ attempt, onEntriesUpdated });

  const messageListContext = useMemo<MessageListContext>(
    () => ({ attempt, task }),
    [attempt, task]
  );

  const computeItemKey = (index: number) => {
    const entry = entries[index];
    return entry ? `l-${entry.patchKey}` : index;
  };

  const renderItem = (_index: number, data: PatchTypeWithKey) => {
    const context = messageListContext;

    if (data.type === 'STDOUT') {
      return <p className="px-4 py-1">{data.content}</p>;
    }
    if (data.type === 'STDERR') {
      return <p className="px-4 py-1 text-red-500">{data.content}</p>;
    }
    if (data.type === 'NORMALIZED_ENTRY' && context?.attempt) {
      return (
        <DisplayConversationEntry
          expansionKey={data.patchKey}
          entry={data.content}
          executionProcessId={data.executionProcessId}
          taskAttempt={context.attempt}
          task={context.task}
        />
      );
    }

    return null;
  };

  return (
    <ApprovalFormProvider>
      <div className="flex-1 flex flex-col h-full relative">
        <Virtuoso<PatchTypeWithKey>
          ref={virtuosoRef}
          className="flex-1"
          data={entries}
          itemContent={renderItem}
          computeItemKey={computeItemKey}
          atBottomStateChange={setAtBottom}
          followOutput={atBottom ? 'smooth' : false}
          increaseViewportBy={{ top: 0, bottom: 600 }}
          initialTopMostItemIndex={entries.length > 0 ? entries.length - 1 : 0}
        />
        {loading && (
          <div className="absolute inset-0 bg-primary/80 flex flex-col gap-2 justify-center items-center z-10">
            <Loader2 className="h-8 w-8 animate-spin" />
            <p>Loading History</p>
          </div>
        )}
      </div>
    </ApprovalFormProvider>
  );
};

export default VirtualizedList;
