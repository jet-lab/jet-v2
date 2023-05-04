import * as Portal from '@radix-ui/react-portal';
import * as Dialog from '@radix-ui/react-dialog';
import { Cross2Icon } from '@radix-ui/react-icons';
import { useCallback, useEffect, useState } from 'react';
import { Title } from './typography';

interface BaseModalProps {
  children: React.ReactNode;
  open: boolean;
  title?: string;
  overlay?: boolean;
  className?: string;
  onClose?: () => void;
}

/**
 * Base modal component
 */
export const Modal = ({ children, title, className, onClose, open, overlay = true }: BaseModalProps) => {
  return (
    <>
      <Portal.Root>
        <Dialog.Root open={open}>
          {overlay && <Dialog.Overlay className="fixed top-0 bottom-0 left-0 right-0 z-10 bg-slate-900 opacity-40" />}
          <Dialog.Content className="fixed top-0 right-0 left-0 bottom-0 z-20 h-screen w-screen">
            <div
              className={`absolute top-1/2 left-1/2 flex -translate-x-1/2 -translate-y-1/2 transform flex-col rounded bg-gradient-to-r from-[#292929] to-[#0E0E0E] p-6 shadow ${
                className ? className : ''
              }`}>
              <Dialog.Close
                asChild
                className="close-modal-button absolute right-3 top-3 flex h-5 w-5 cursor-pointer items-center justify-center rounded-sm bg-neutral-700"
                aria-label="Close"
                onClick={onClose}>
                <Cross2Icon />
              </Dialog.Close>
              {title && <Title classNameOverride="mr-8">{title}</Title>}
              {children}
            </div>
          </Dialog.Content>
        </Dialog.Root>
      </Portal.Root>
    </>
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
export const DismissModal = ({ children, storageKey, title, className }: DismissModalProps) => {
  const [open, setOpen] = useState<boolean>(false);

  useEffect(() => {
    const dismissedDate = localStorage.getItem(storageKey);
    !dismissedDate && setOpen(true);
  }, []);

  const dismiss = useCallback(() => {
    localStorage.setItem(storageKey, new Date().toUTCString());
    setOpen(false);
  }, [storageKey]);

  return (
    <Modal open={open} title={title} className={className} onClose={() => setOpen(false)}>
      {children({
        dismiss
      })}
    </Modal>
  );
};
