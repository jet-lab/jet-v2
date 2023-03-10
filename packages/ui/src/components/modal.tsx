import * as Portal from '@radix-ui/react-portal'
import * as Dialog from '@radix-ui/react-dialog'
import { Cross2Icon } from '@radix-ui/react-icons';
import { useEffect, useState } from 'react';

interface BaseModalProps {
    children: React.ReactNode
    open: boolean

}

export const Modal = ({ children, open }: BaseModalProps) => {
    console.log('modal')
    return <Portal.Root>
        <Dialog.Root defaultOpen={true} open={open}>
            <Dialog.Overlay />
            <Dialog.Content>
                <div className='p-4 absolute top-0 left-0 w-96 h-96 bg-red-500'>
                    {children}
                    <Dialog.Close asChild>
                        <button aria-label="Close">
                            <Cross2Icon />
                        </button>
                    </Dialog.Close>
                </div>
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