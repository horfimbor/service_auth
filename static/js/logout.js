class Logout extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `<button class="logout">Logout</button>`

    }

    connectedCallback() {
        this.shadowRoot.querySelector(".logout").addEventListener('click', this._logout.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector(".logout").removeEventListener('click', this._logout);
    }

    _logout(e){
        e.preventDefault();
        var event = new CustomEvent('logout');
//        this.dispatchEvent(event);
//        this.shadowRoot.dispatchEvent(event);
        document.dispatchEvent(event);
    }

}
customElements.define('hf-auth-logout', Logout);