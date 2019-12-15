class Logout extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `
            <hf-button content="logout" class="logout"></hf-button>
        `

    }

    connectedCallback() {
        this.shadowRoot.querySelector(".logout").addEventListener('click', this._logout.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector(".logout").removeEventListener('click', this._logout);
    }

    _logout(e){
        e.preventDefault();
        var event = new CustomEvent('_auth_logout');
        document.dispatchEvent(event);
    }

}
customElements.define('hf-auth-logout', Logout);