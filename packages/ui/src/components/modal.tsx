import * as Portal from '@radix-ui/react-portal'
import * as Dialog from '@radix-ui/react-dialog'
import { Cross2Icon } from '@radix-ui/react-icons';
import { useEffect, useState } from 'react';

interface BaseModalProps {
    children: React.ReactNode
    open?: boolean

}

export const Modal = ({ children, open }: BaseModalProps) => {
    return <Portal.Root className='h-screen w-screen absolute top-0 right-0 left-0 bottom-0'>
        <Dialog.Root defaultOpen={true} open={open}>
            <Dialog.Overlay className="opacity-40 bg-black absolute top-0 bottom-0 left-0 right-0 z-10" />
            <Dialog.Content className='p-4 absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-gradient-to-r from-neutral-800 to-neutral-900 z-20 flex rounded-sm shadow'>
                <Dialog.Close asChild className='cursor-pointer bg-gray-800 rounded-sm h-6 w-6 flex items-center justify-center' aria-label="Close">
                    <Cross2Icon />
                </Dialog.Close>
                {children}
            </Dialog.Content>
        </Dialog.Root>
    </Portal.Root>
}

interface OneOffModalProps extends BaseModalProps {
    storageKey: string
}

export const OneOffModal = ({ children, storageKey }: OneOffModalProps) => {
    const [open, setOpen] = useState(false)

    useEffect(() => {
        const dismissedDate = localStorage.getItem(storageKey)
        dismissedDate && setOpen(true)
    }, [])

    return <Modal open={open}>{children}</Modal>
}