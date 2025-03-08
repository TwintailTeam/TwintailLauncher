import React from 'react'
import {open} from "@tauri-apps/plugin-dialog"
import './FolderInput.css'
import TextInputPart from "./TextInputPart.tsx";

// Thanks Cultivation FUCK NO im not making this myself
// Yes I can not be assed to make inputs, I stole Cultivation's and modified them to fit the theme

interface IProps {
    value?: string
    clearable?: boolean
    onChange?: (value: string) => void
    extensions?: string[]
    readonly?: boolean
    placeholder?: string
    folder?: boolean
    customClearBehaviour?: () => void
    openFolder?: string,
    name?: string,
    id?: string,
}

interface IState {
    value: string
    placeholder: string
    folder: boolean
}

export default class FolderInput extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props)

        this.state = {
            value: props.value || '',
            placeholder: this.props.placeholder || 'Select file or folder...',
            folder: this.props.folder || false,
        }

        this.handleIconClick = this.handleIconClick.bind(this)
    }

    static getDerivedStateFromProps(props: IProps, state: IState) {
        const newState = state

        if (props.value && state.value === '') {
            newState.value = props.value || ''
        }

        if (props.placeholder) {
            newState.placeholder = props.placeholder
        }

        return newState
    }

    async componentDidMount() {
        if (!this.props.placeholder) {
            this.setState({placeholder: "Select folder..."})
        }
    }

    async handleIconClick() {
        let path

        if (this.state.folder) {
            path = await open({directory: true})
        } else {
            path = await open({filters: [{ name: 'Files', extensions: this.props.extensions || ['*'] }], defaultPath: this.props.openFolder})
        }

        if (Array.isArray(path)) path = path[0]
        if (!path) return

        this.setState({
            value: path,
        })

        if (this.props.onChange) this.props.onChange(path)
    }

    render() {
        return (
            <div className="flex flex-row items-center justify-between w-full h-6">
                <span className="text-white text-sm">{this.props.name}</span>
                <div className="DirInput">
                    <TextInputPart value={this.state.value}
                                   isPicker={true}
                                   onClick={this.handleIconClick}
                                   placeholder={this.state.placeholder}
                                   clearable={this.props.clearable !== undefined ? this.props.clearable : true}
                                   readOnly={this.props.readonly !== undefined ? this.props.readonly : true}
                                   onChange={(text: string) => {
                                   this.setState({ value: text })
                                   if (this.props.onChange) this.props.onChange(text)
                                   this.forceUpdate()
                               }} customClearBehaviour={this.props.customClearBehaviour}/>
                </div>
            </div>
        )
    }
}
