import * as Portal from '@radix-ui/react-portal';
import * as Dialog from '@radix-ui/react-dialog';
import { Cross2Icon } from '@radix-ui/react-icons';
import { useCallback, useEffect, useState } from 'react';
import { Title } from './typography';

interface BaseModalProps {
  children: React.ReactNode;
  open?: boolean;
  title?: string;
  overlay?: boolean;
}

/**
 * Base modal component
 */
export const Modal = ({ children, open, title, overlay = true }: BaseModalProps) => {
  return (
    <Portal.Root className="absolute top-0 right-0 left-0 bottom-0 h-screen w-screen">
      <Dialog.Root defaultOpen={true} open={open}>
        {overlay && <Dialog.Overlay className="absolute top-0 bottom-0 left-0 right-0 z-10 bg-slate-900 opacity-40" />}
        <Dialog.Content className="absolute top-1/2 left-1/2 z-20 flex -translate-x-1/2 -translate-y-1/2 transform flex-col rounded bg-gradient-to-r from-[#292929] to-[#0E0E0E] p-6 shadow">
          <Dialog.Close
            asChild
            className="absolute right-3 top-3 flex h-5 w-5 cursor-pointer items-center justify-center rounded-sm bg-neutral-700"
            aria-label="Close">
            <Cross2Icon />
          </Dialog.Close>
          {title && <Title classNameOverride="mr-8" text={title} />}
          {children}
        </Dialog.Content>
      </Dialog.Root>
    </Portal.Root>
  );
};

interface DismissModalProps extends Omit<BaseModalProps, 'children'> {
  storageKey: string;
  children: (args: { dismiss: () => void }) => React.ReactNode;
}

/**
 * Variant of the modal component with localStorage caching. Useful for modals that can be dismissed to never be shown again.
 * Call dismiss to never show this modal to a user
 * Sample Usage:
 * ```
 * <DismissModal storageKey='any-string' title="Your Title">
 *   {({ dismiss }) => <div onClick={dismiss}>Your Content</div>}
 * </DismissModal>
 * ```
 */
export const DismissModal = ({ children, storageKey, title }: DismissModalProps) => {
  const [open, setOpen] = useState<boolean | undefined>(false);

  const dismiss = useCallback(() => {
    localStorage.setItem(storageKey, new Date().toUTCString());
    setOpen(false);
  }, [storageKey]);

  useEffect(() => {
    const dismissedDate = localStorage.getItem(storageKey);
    !dismissedDate && setOpen(undefined);
  }, []);

  return (
    <Modal title={title} open={open}>
      {children({
        dismiss
      })}
    </Modal>
  );
};