import React from 'react'

import './TextInputPart.css'

// Thanks Cultivation
// Yes I can not be assed to make inputs, I stole Cultivation's and modified them to fit the theme

interface IProps {
    value?: string
    initalValue?: string
    placeholder?: string
    onChange?: (value: string) => void
    readOnly?: boolean
    id?: string
    clearable?: boolean
    customClearBehaviour?: () => void
    onClick?: () => void
    style?: React.CSSProperties
    isPicker?: boolean
}

interface IState {
    value: string
}

export default class TextInputPart extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props)

        this.state = {
            value: props.value || '',
        }
    }

    async componentDidMount() {
        if (this.props.initalValue) {
            this.setState({
                value: this.props.initalValue,
            })
        }
    }

    static getDerivedStateFromProps(props: IProps, state: IState) {
        return { value: props.value || state.value }
    }

    render() {
        return (
            <div className="TextInputWrapper" style={this.props.style || {}}>
                <input id={this.props?.id}
                    readOnly={this.props.readOnly || false}
                    placeholder={this.props.placeholder || ''}
                    className={`TextInput w-full relative rounded-full transition-all bg-white/10 text-white indent-2.5 ${this.props.isPicker ? "cursor-pointer" : ""}`}
                    value={this.state.value}
                    onChange={(e) => {
                        this.setState({ value: e.target.value })
                        if (this.props.onChange) this.props.onChange(e.target.value)
                    }} onClick={() => {
                        if (this.props.onClick) this.props.onClick();
                }}
                />
                {this.props.clearable ? (
                    <div className="TextClear"
                        onClick={() => {
                            // Run custom behaviour first
                            if (this.props.customClearBehaviour) return this.props.customClearBehaviour()
                            this.setState({ value: '' })
                            if (this.props.onChange) this.props.onChange('')
                            this.forceUpdate()
                        }}>
                    </div>
                ) : null}
            </div>
        )
    }
}
