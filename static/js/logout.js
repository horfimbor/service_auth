class Logout extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `<button class="logout">Logout</button>`
    }

    connectedCallback() {
        this.shadowRoot.querySelector(".logout").addEventListener('submit', this._logout.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector(".logout").removeEventListener('submit', this._logout);
    }

}
customElements.define('hf-auth-logout', Login);