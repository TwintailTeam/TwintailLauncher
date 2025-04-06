import React from 'react'

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
    isPicker?: boolean,
    pattern?: string
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
            <div className="w-full" style={this.props.style || {}}>
                <input id={this.props?.id}
                    readOnly={this.props.readOnly || false}
                    placeholder={this.props.placeholder || ''}
                    className={`text-ellipsis w-full focus:outline-none h-8 rounded-lg bg-white/20 text-white px-2 placeholder-white/50 ${this.props.isPicker ? "cursor-pointer" : ""}`}
                    value={this.state.value}
                       pattern={this.props.pattern || ""}
                    onChange={(e) => {
                        this.setState({ value: e.target.value })
                        if (this.props.onChange) this.props.onChange(e.target.value)
                    }} onClick={() => {
                        if (this.props.onClick) this.props.onClick();
                }}/>
                {this.props.clearable ? (
                    <div className="hidden"
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
